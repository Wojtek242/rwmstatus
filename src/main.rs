//! # rwmstatus
//!
//! Status monitor bar for the dwm/rwm window manager (or any WM which uses
//! WM_NAME of the X11 root window as its status bar).  It is a direct port of
//! [dwmstatus](https://dwm.suckless.org/status_monitor/) to Rust.
//!
//! This is part of a larger project to port various
//! [suckless.org](https://suckless.org/) programs to Rust, a programming
//! language that sucks less.

// Lib imports
extern crate rwmstatus;
use rwmstatus::*;

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

        let status = CString::new(format!("T:{} L:{} B:{} {}", temps, avgs, batts, times))
            .expect("Failed to create status CString.");
        unsafe {
            XStoreName(display, XDefaultRootWindow(display), status.as_ptr());
            XSync(display, false as i32);
        }

        thread::sleep(time::Duration::from_secs(60));
    }
}
