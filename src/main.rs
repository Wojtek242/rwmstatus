//! # rwmstatus
//!
//! Status monitor bar for the dwm/rwm window manager (or any WM which uses
//! WM_NAME of the X11 root window as its status bar).  It is a direct port of
//! [dwmstatus](https://dwm.suckless.org/status_monitor/) to Rust.
//!
//! This is part of a larger project to port various
//! [suckless.org](https://suckless.org/) programs to Rust, a programming
//! language that sucks less.

extern crate chrono;
extern crate chrono_tz;
extern crate libc;
extern crate x11;

// std module imports
use std::env;
use std::io;
use std::process;
use std::ptr;
use std::thread;
use std::time;

// std type imports
use std::io::prelude::*;
use std::ffi::CString;
use std::fs::File;

// x11 imports
use x11::xlib::Display;
use x11::xlib::{XDefaultRootWindow, XOpenDisplay, XStoreName, XSync};

// Other external imports
use chrono::prelude::*;

// Internal module imports
mod config;
use config::*;

/// Convert a Rust string to a CString and panic if it fails.
#[inline]
fn cstring(string: &str) -> CString {
    CString::new(string).expect(&format!("CString::new({}) failed.", string))
}

/// Read the contents of the file base/filename and return as a String.
#[inline]
fn read_file(base: &str, filename: &str) -> io::Result<String> {
    let mut file = File::open([base, filename].join("/"))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

/// Return temperature reads from all monitors.
fn get_temperatures() -> String {
    let mut temp_strs: Vec<String> = vec![];

    for hwmon in HWMONS.iter() {
        temp_strs.push(get_temp(&[HWMON_PATH, hwmon].join("/")));
    }

    temp_strs.join("|")
}

/// Return temperature read from the provided monitor.
fn get_temp(hwmon: &str) -> String {
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
fn get_load_avgs() -> String {
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

/// Return battery status for all batteries.
fn get_batteries() -> String {
    let mut batt_strs: Vec<String> = vec![];

    for batt in BATTS.iter() {
        batt_strs.push(get_batt(&[BATT_PATH, batt].join("/")));
    }

    batt_strs.join("|")
}

/// Return battery status for the battery at the provided path.
fn get_batt(batt: &str) -> String {
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

/// Return times for all configured time zones.
fn get_times() -> String {
    let mut tz_strs: Vec<String> = vec![];

    for tz in TZS.iter() {
        tz_strs.push(format!("{}:{}", tz.0, get_tz_time(tz.1, "%H:%M")));
    }

    tz_strs.push(get_tz_time(TZ_DEF, "KW %W %a %d %b %H:%M %Z %Y"));

    tz_strs.join(" ")
}

/// Get the time for the provided time zone.
fn get_tz_time(tz_name: &str, fmt: &str) -> String {
    match tz_name.parse::<chrono_tz::Tz>() {
        Ok(tz) => {
            let utc = Utc::now().naive_utc();
            format!("{}", tz.from_utc_datetime(&utc).format(fmt))
        }
        Err(_) => return format!(""),
    }
}

fn main() {
    let display: *mut Display;

    unsafe {
        display = XOpenDisplay(ptr::null());
    }

    if display == ptr::null_mut() {
        eprintln!("rwmstatus: cannot open display.");
        process::exit(1);
    }

    loop {
        let temps = get_temperatures();
        let avgs = get_load_avgs();
        let batts = get_batteries();
        let times = get_times();

        let status = cstring(&format!("T:{} L:{} B:{} {}", temps, avgs, batts, times));
        unsafe {
            XStoreName(display, XDefaultRootWindow(display), status.as_ptr());
            XSync(display, false as i32);
        }

        thread::sleep(time::Duration::from_secs(60));
    }
}
