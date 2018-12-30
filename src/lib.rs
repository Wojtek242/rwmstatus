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

// std imports
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

// External imports
use chrono::prelude::*;

/// Return temperature read from the provided monitor.
pub fn get_temp(hwmon: &PathBuf) -> Result<String> {
    let val: i64 = read_to_string(hwmon.join("temp1_input"))?.trim().parse()?;
    Ok(format!("{:02}°C", val / 1000))
}

/// Return the three load average values.
pub fn get_load_avgs() -> Result<String> {
    let mut avgs: [libc::c_double; 3] = [0.0; 3];

    let rc = unsafe { libc::getloadavg(&mut avgs[0] as *mut libc::c_double, 3) };
    if rc < 0 {
        return Err(StatusError::System(rc));
    }

    Ok(format!("{:.2} {:.2} {:.2}", avgs[0], avgs[1], avgs[2]))
}

/// Return battery status for the battery at the provided path.
pub fn get_batt(batt: &PathBuf) -> Result<String> {
    if !read_to_string(batt.join("present"))?.starts_with('1') {
        return Err(StatusError::NotPresent(batt.to_str().unwrap().to_string()));
    }

    let desired_capacity: u64 = read_to_string(batt.join("charge_full_design"))
        .or_else(|_| read_to_string(batt.join("energy_full_design")))?
        .trim()
        .parse()?;

    let remaining_capacity: u64 = read_to_string(batt.join("charge_now"))
        .or_else(|_| read_to_string(batt.join("energy_now")))?
        .trim()
        .parse()?;

    let status: char = match read_to_string(batt.join("status")) {
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

    let percentage = ((remaining_capacity as f64) / (desired_capacity as f64)) * 100.0;
    Ok(format!("{:.0}%{}", percentage, status))
}

/// Get the time for the provided time zone.
pub fn get_tz_time(tz_name: &str, fmt: &str) -> Result<String> {
    let tz: chrono_tz::Tz = tz_name.parse().map_err(StatusError::ParseTz)?;
    let utc = Utc::now().naive_utc();
    Ok(format!("{}", tz.from_utc_datetime(&utc).format(fmt)))
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
            Ok(path) => {
                match path.file_name().to_str() {
                    Some(entry) => entry.starts_with(prefix),
                    None => false,
                }
            }
            Err(_) => false,
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
    pub fn get_temperatures(&self) -> Option<String> {
        if self.hw_mons.is_empty() {
            return None;
        }

        let temp_strs: Vec<String> = self.hw_mons
            .iter()
            .map(|hw_mon| get_temp(&hw_mon).unwrap_or("".into()))
            .collect();
        Some(temp_strs.join("|"))
    }

    /// Return the three load average values.
    #[inline]
    pub fn get_load_avgs(&self) -> String {
        get_load_avgs().unwrap_or(format!(""))
    }

    /// Return battery status for all batteries.
    pub fn get_batteries(&self) -> Option<String> {
        if self.batts.is_empty() {
            return None;
        }

        let batt_strs: Vec<String> = self.batts
            .iter()
            .map(|batt| get_batt(&batt).unwrap_or("".into()))
            .collect();
        Some(batt_strs.join("|"))
    }

    /// Return times for all configured time zones.
    pub fn get_times(&self) -> String {
        let mut tz_strs: Vec<String> = self.tzs
            .iter()
            .map(|tz| {
                format!(
                    "{}:{}",
                    tz.label,
                    get_tz_time(&tz.name, "%H:%M").unwrap_or("".into())
                )
            })
            .collect();
        tz_strs.push(get_local_time("KW %W %a %d %b %H:%M %Z %Y"));
        tz_strs.join(" ")
    }
}

/// Internal `Result` type.
type Result<T> = std::result::Result<T, StatusError>;

/// Error type for `rwmstatus` functions.
#[derive(Debug)]
pub enum StatusError {
    Io(std::io::Error),
    ParseNum(std::num::ParseIntError),
    ParseTz(String),
    NotPresent(String),
    System(i32),
}

impl std::fmt::Display for StatusError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            StatusError::Io(ioe) => ioe.fmt(f),
            StatusError::ParseNum(pie) => pie.fmt(f),
            StatusError::ParseTz(s) => write!(f, "{}", s),
            StatusError::NotPresent(s) => write!(f, "{} not present", s),
            StatusError::System(i) => write!(f, "System call returned {}", i),
        }
    }
}

impl std::error::Error for StatusError {
    fn description(&self) -> &str {
        match self {
            StatusError::Io(ioe) => ioe.description(),
            StatusError::ParseNum(pie) => pie.description(),
            StatusError::ParseTz(_) => "Invalid timezone",
            StatusError::NotPresent(_) => "Device not present",
            StatusError::System(_) => "System call returned error",
        }
    }
}

impl From<std::io::Error> for StatusError {
    fn from(err: std::io::Error) -> Self {
        StatusError::Io(err)
    }
}

impl From<std::num::ParseIntError> for StatusError {
    fn from(err: std::num::ParseIntError) -> Self {
        StatusError::ParseNum(err)
    }
}
