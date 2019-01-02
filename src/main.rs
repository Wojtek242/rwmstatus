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

// std imports
use std::ffi::CString;

// x11 imports
use x11::xlib::{Display, XDefaultRootWindow, XOpenDisplay, XStoreName, XSync};

// Internal module imports
mod config;

fn main() {
    let display: *mut Display;

    unsafe {
        display = XOpenDisplay(std::ptr::null());
    }

    if display.is_null() {
        eprintln!("rwmstatus: cannot open display.");
        std::process::exit(1);
    }

    let rwmstatus = RwmStatus::new(&config::TZS[..]);

    let mut stats = vec![];
    loop {
        if let Some(temps) = rwmstatus.get_temperatures() {
            stats.push(format!("T:{}", temps));
        }

        let avgs = rwmstatus.get_load_avgs();
        stats.push(format!("L:{}", avgs));

        if let Some(batts) = rwmstatus.get_batteries() {
            stats.push(format!("B:{}", batts));
        }

        let times = rwmstatus.get_times();
        stats.push(times);

        let status = CString::new(stats.join(" ")).expect("Failed to create status CString.");
        unsafe {
            XStoreName(display, XDefaultRootWindow(display), status.as_ptr());
            XSync(display, false as i32);
        }

        std::thread::sleep(std::time::Duration::from_secs(60));

        stats.clear();
    }
}
