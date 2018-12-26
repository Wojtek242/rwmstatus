//! # rwmstatus configuration

/// Path to monitors.
pub const HW_MON_PATH: &'static str = "/sys/devices/virtual/hwmon";

/// Path to power supply information.
pub const BATT_PATH: &'static str = "/sys/class/power_supply";

/// Additional time zones to display (short name, full name).
pub const TZS: [(char, &'static str); 2] = [('A', "America/Buenos_Aires"), ('U', "UTC")];
