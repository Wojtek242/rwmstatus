extern crate x11;

use x11::xlib::Display;
use x11::xlib::{XDefaultRootWindow, XOpenDisplay, XStoreName, XSync};

use std::ptr;
use std::process;
use std::thread;
use std::time;

use std::ffi::CString;

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
        let status = CString::new("Hello!").expect("CString::new failed when setting status.");

        unsafe {
            XStoreName(display, XDefaultRootWindow(display), status.as_ptr());
            XSync(display, false as i32);
        }

        thread::sleep(time::Duration::from_secs(1));
    }
}
