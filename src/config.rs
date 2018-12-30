//! # rwmstatus configuration

/// Path to monitors.
pub const HW_MON_PATH: &str = "/sys/devices/virtual/hwmon";

/// Path to power supply information.
pub const BATT_PATH: &str = "/sys/class/power_supply";

/// Additional time zones to display (short name, full name).
pub const TZS: [(char, &str); 2] = [('A', "America/Buenos_Aires"), ('U', "UTC")];
