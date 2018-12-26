rwmstatus
=========

[![Latest version](https://img.shields.io/crates/v/rwmstatus.svg)](https://crates.io/crates/rwmstatus)
[![Documentation](https://docs.rs/rwmstatus/badge.svg)](https://docs.rs/rwmstatus)
![License](https://img.shields.io/crates/l/rwmstatus.svg)

Library for status monitor displays for window managers such as dwm/rwm.  It is
a direct port of [dwmstatus](https://dwm.suckless.org/status_monitor/) to Rust.

This crate is intended to be used as either a library that provides various
utility functions for obtaining system readouts or as a standalone binary
together with a window manager like dwm or rwm.

The standalone binary's
[main.rs](https://github.com/Wojtek242/rwmstatus/blob/master/src/main.rs) shows
an example of how to use the library.