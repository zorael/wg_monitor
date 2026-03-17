//! Logging utilities.

use crate::utils;

/// Prints a timestamp prefix in the format "[HH:MM:SS] " to standard output.
pub fn print_timestamp_prefix() {
    print!("[{}] ", utils::timestamp_now());
}

/// Prints a timestamp prefix in the format "[HH:MM:SS] " to standard error.
pub fn eprint_timestamp_prefix() {
    eprint!("[{}] ", utils::timestamp_now());
}

/// Prints a timestamped message to standard out if timestamps are enabled in
/// settings, otherwise just prints the message.
//#[macro_export]
macro_rules! tsprintln {
    ($settings:expr, $($args:tt)*) => {{
        if !$settings.disable_timestamps {
            $crate::logging::print_timestamp_prefix();
        }
        println!($($args)*);
    }};
}

/// Prints a timestamped message to standard error if timestamps are enabled in
/// settings, otherwise just prints the message.
//#[macro_export]
macro_rules! tseprintln {
    ($settings:expr, $($args:tt)*) => {{
        if !$settings.disable_timestamps {
            $crate::logging::eprint_timestamp_prefix();
        }
        eprintln!($($args)*);
    }};
}

// Re-export macros for use in other modules.
pub(crate) use tseprintln;
pub(crate) use tsprintln;
