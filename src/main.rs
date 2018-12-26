//! # rwmstatus
//!
//! Status monitor bar binary for the dwm/rwm window manager (or any WM which
//! uses WM_NAME of the X11 root window as its status bar).  It is a direct
//! port of [dwmstatus](https://dwm.suckless.org/status_monitor/) to Rust.
//!
//! This is part of my [Rust Sucks
//! Less](https://wojciechkozlowski.eu/rust-sucks-less/) project to port some
//! of the [suckless.org](https://suckless.org/) programs and tools to Rust, a
//! programming language that sucks less.

// Lib import
extern crate rwmstatus;
use rwmstatus::*;

// External crates
extern crate x11;

// std module imports
use std::process;
use std::ptr;
use std::thread;
use std::time;

// std type imports
use std::ffi::CString;

// x11 imports
use x11::xlib::Display;
use x11::xlib::{XDefaultRootWindow, XOpenDisplay, XStoreName, XSync};

// Internal module imports
mod config;

fn main() {
    let display: *mut Display;

    unsafe {
        display = XOpenDisplay(ptr::null());
    }

    if display == ptr::null_mut() {
        eprintln!("rwmstatus: cannot open display.");
        process::exit(1);
    }

    let rwmstatus = RwmStatus::new(config::HW_MON_PATH, config::BATT_PATH, &config::TZS[..]);

    loop {
        let mut stats = vec![];

        let temps = rwmstatus.get_temperatures();
        if !temps.is_empty() {
            stats.push(format!("T:{}", temps));
        }

        let avgs = rwmstatus.get_load_avgs();
        if !avgs.is_empty() {
            stats.push(format!("L:{}", avgs));
        }

        let batts = rwmstatus.get_batteries();
        if !batts.is_empty() {
            stats.push(format!("B:{}", batts));
        }

        let times = rwmstatus.get_times();
        if !times.is_empty() {
            stats.push(times);
        }

        let status = CString::new(stats.join(" ")).expect("Failed to create status CString.");
        unsafe {
            XStoreName(display, XDefaultRootWindow(display), status.as_ptr());
            XSync(display, false as i32);
        }

        thread::sleep(time::Duration::from_secs(60));
    }
}
