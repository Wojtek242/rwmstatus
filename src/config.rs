//! # rwmstatus configuration

/// Path to monitors.
pub const HWMON_PATH: &'static str = "/sys/devices/virtual/hwmon";

/// Array of monitors to use.
pub const HWMONS: [&'static str; 3] = ["hwmon0", "hwmon2", "hwmon4"];

/// Path to power supply information.
pub const BATT_PATH: &'static str = "/sys/class/power_supply";

/// Batteries to display.
pub const BATTS: [&'static str; 2] = ["BAT0", "BAT1"];

/// Additional time zones to display (short name, full name).
pub const TZS: [(char, &'static str); 2] = [('A', "America/Buenos_Aires"), ('U', "UTC")];
