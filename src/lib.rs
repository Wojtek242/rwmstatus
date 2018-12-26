//! # rwmstatus
//!
//! Library for status monitor displays.  It provides functions to obtain
//! readouts about system status such as battery status or temperature.
//!
//! This is part of my [Rust Sucks
//! Less](https://wojciechkozlowski.eu/rust-sucks-less/) project to port some
//! of the [suckless.org](https://suckless.org/) programs and tools to Rust, a
//! programming language that sucks less.

// External crates
extern crate chrono;
extern crate chrono_tz;
extern crate libc;

// std module imports
use std::io;

// std type imports
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

// Other external imports
use chrono::prelude::*;

/// Read the contents of the file base/filename and return as a String.
#[inline]
fn read_file(base: &PathBuf, filename: &str) -> io::Result<String> {
    let mut file = File::open(base.join(filename))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

/// Return temperature read from the provided monitor.
pub fn get_temp(hwmon: &PathBuf) -> String {
    match read_file(hwmon, "temp1_input") {
        Ok(contents) => {
            match contents.trim().parse::<f64>() {
                Ok(val) => format!("{:02.0}°C", val / 1000.0),
                Err(_) => format!(""),
            }
        }
        Err(_) => format!(""),
    }
}

/// Return the three load average values.
pub fn get_load_avgs() -> String {
    let mut avgs: [libc::c_double; 3] = [0.0; 3];

    let rc: libc::c_int;
    unsafe {
        rc = libc::getloadavg(&mut avgs[0] as *mut libc::c_double, 3);
    }

    if rc < 0 {
        return format!("");
    }

    format!("{:.2} {:.2} {:.2}", avgs[0], avgs[1], avgs[2])
}

/// Return battery status for the battery at the provided path.
pub fn get_batt(batt: &PathBuf) -> String {
    match read_file(&batt, "present") {
        Ok(contents) => {
            if !contents.starts_with('1') {
                return format!("not present");
            }
        }
        Err(_) => return format!(""),
    };

    let co = match read_file(&batt, "charge_full_design") {
        Ok(contents) => contents,
        Err(_) => {
            match read_file(&batt, "energy_full_design") {
                Ok(contents) => contents,
                Err(_) => return format!(""),
            }
        }
    };

    let desired_capacity: u64 = match co.trim().parse() {
        Ok(val) => val,
        Err(_) => return format!("invalid"),
    };

    let co = match read_file(&batt, "charge_now") {
        Ok(contents) => contents,
        Err(_) => {
            match read_file(&batt, "energy_now") {
                Ok(contents) => contents,
                Err(_) => return format!(""),
            }
        }
    };

    let remaining_capacity: u64 = match co.trim().parse() {
        Ok(val) => val,
        Err(_) => return format!("invalid"),
    };

    let status: char = match read_file(&batt, "status") {
        Ok(contents) => {
            match &contents.trim()[..] {
                "Full" => 'F',
                "Discharging" => '-',
                "Charging" => '+',
                _ => '?',
            }
        }
        Err(_) => '?',
    };

    format!(
        "{:.0}%{}",
        ((remaining_capacity as f64) / (desired_capacity as f64)) * 100.0,
        status
    )
}

/// Get the time for the provided time zone.
pub fn get_tz_time(tz_name: &str, fmt: &str) -> String {
    match tz_name.parse::<chrono_tz::Tz>() {
        Ok(tz) => {
            let utc = Utc::now().naive_utc();
            format!("{}", tz.from_utc_datetime(&utc).format(fmt))
        }
        Err(_) => return format!(""),
    }
}

/// Get the local time.
pub fn get_local_time(fmt: &str) -> String {
    format!("{}", Local::now().format(fmt))
}

/// ## RwmStatus
///
/// This struct collects device dependent paths and user settings.  It also
/// provides convenience methods to aggregate readouts.
pub struct RwmStatus {
    hw_mons: Vec<PathBuf>,
    batts: Vec<PathBuf>,
    tzs: Vec<Tz>,
}

/// ## Tz
///
/// Holds the label and name of a time zone.
struct Tz {
    label: char,
    name: String,
}

impl RwmStatus {
    /// Build a new RwmStatus object.  This function collects all the monitor
    /// and battery paths for later use.
    pub fn new(hw_mon_path: &str, batt_path: &str, tzs: &[(char, &str)]) -> RwmStatus {
        RwmStatus {
            hw_mons: RwmStatus::get_paths(hw_mon_path, "hwmon"),
            batts: RwmStatus::get_paths(batt_path, "BAT"),
            tzs: tzs.iter()
                .map(|tz| {
                    Tz {
                        label: tz.0,
                        name: String::from(tz.1),
                    }
                })
                .collect(),
        }
    }

    /// Collect all the paths of the form base_path/prefix*
    fn get_paths(base_path: &str, prefix: &str) -> Vec<PathBuf> {
        let dir = match Path::new(base_path).read_dir() {
            Ok(iter) => iter,
            Err(_) => return vec![],
        };

        let dir_filtered = dir.filter(|path_result| match path_result {
            &Ok(ref path) => {
                match path.file_name().to_str() {
                    Some(entry) => entry.starts_with(prefix),
                    None => false,
                }
            }
            &Err(_) => false,
        });

        let mut paths: Vec<PathBuf> = dir_filtered
            .map(|path_result| match path_result {
                Ok(path) => path.path(),
                Err(_) => panic!("Unexpected file path"),
            })
            .collect();

        paths.sort_unstable();
        paths
    }

    /// Return temperature reads from all monitors.
    pub fn get_temperatures(&self) -> String {
        let mut temp_strs: Vec<String> = vec![];

        for hwmon in self.hw_mons.iter() {
            temp_strs.push(get_temp(&hwmon));
        }

        temp_strs.join("|")
    }

    /// Return the three load average values.
    #[inline]
    pub fn get_load_avgs(&self) -> String {
        get_load_avgs()
    }

    /// Return battery status for all batteries.
    pub fn get_batteries(&self) -> String {
        let mut batt_strs: Vec<String> = vec![];

        for batt in self.batts.iter() {
            batt_strs.push(get_batt(&batt));
        }

        batt_strs.join("|")
    }

    /// Return times for all configured time zones.
    pub fn get_times(&self) -> String {
        let mut tz_strs: Vec<String> = vec![];

        for tz in self.tzs.iter() {
            tz_strs.push(format!("{}:{}", tz.label, get_tz_time(&tz.name, "%H:%M")));
        }

        tz_strs.push(get_local_time("KW %W %a %d %b %H:%M %Z %Y"));

        tz_strs.join(" ")
    }
}
