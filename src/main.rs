//! # rwmstatus
//!
//! Status monitor bar for the dwm/rwm window manager (or any WM which uses
//! WM_NAME of the X11 root window as its status bar).  It is a direct port of
//! [dwmstatus](https://dwm.suckless.org/status_monitor/) to Rust.
//!
//! This is part of a larger project to port various
//! [suckless.org](https://suckless.org/) programs to Rust, a programming
//! language that sucks less.

extern crate x11;
extern crate libc;

// std module imports
use std::io;
use std::io::prelude::*;
use std::process;
use std::ptr;
use std::thread;
use std::time;

// std type imports
use std::ffi::CString;
use std::fs::File;

// x11 imports
use x11::xlib::Display;
use x11::xlib::{XDefaultRootWindow, XOpenDisplay, XStoreName, XSync};

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

const BATT_PATH: &'static str = "/sys/class/power_supply";
const BATTS: [&'static str; 2] = ["BAT0", "BAT1"];

/// Return battery status for all batteries.
fn get_batteries() -> String {
    let mut batt_strs: Vec<String> = vec![];

    for batt in BATTS.iter() {
        batt_strs.push(get_battery(&[BATT_PATH, batt].join("/")));
    }

    batt_strs.join("|")
}

/// Return battery status for the battery at the provided path.
fn get_battery(batt_base: &str) -> String {
    match read_file(&batt_base, "present") {
        Ok(contents) => {
            if !contents.starts_with('1') {
                return format!("not present");
            }
        }
        Err(_) => return format!(""),
    };

    let co = match read_file(&batt_base, "charge_full_design") {
        Ok(contents) => contents,
        Err(_) => {
            match read_file(&batt_base, "energy_full_design") {
                Ok(contents) => contents,
                Err(_) => return format!(""),
            }
        }
    };

    let desired_capacity: u64 = co.trim().parse().expect(&format!("Not a number: {}", co));

    let co = match read_file(&batt_base, "charge_now") {
        Ok(contents) => contents,
        Err(_) => {
            match read_file(&batt_base, "energy_now") {
                Ok(contents) => contents,
                Err(_) => return format!(""),
            }
        }
    };

    let remaining_capacity: u64 = co.trim().parse().expect(&format!("Invalid number: {}", co));

    let status: char = match read_file(&batt_base, "status") {
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
        let avgs = get_load_avgs();
        let batts = get_batteries();

        let status = cstring(&format!("L:{} B:{}", avgs, batts));
        unsafe {
            XStoreName(display, XDefaultRootWindow(display), status.as_ptr());
            XSync(display, false as i32);
        }

        thread::sleep(time::Duration::from_secs(60));
    }
}
