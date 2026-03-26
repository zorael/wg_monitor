//! Logging utilities for the WireGuard monitor program.
//!
//! This is just macro magic.

use crate::utils;

/// Prints a timestamp prefix in the format "[HH:MM:SS] " to standard out.
pub fn print_timestamp_prefix() {
    print!("[{}] ", utils::timestamp_now());
}

/// Prints a timestamp prefix in the format "[HH:MM:SS] " to standard error.
pub fn eprint_timestamp_prefix() {
    eprint!("[{}] ", utils::timestamp_now());
}

/// Prints a timestamped message to standard out if the passed
/// `disable_timestamps` flag is `false`, otherwise just prints the message.
///
/// The message is formatted using the standard `println!` macro, and the
/// timestamp is printed as a prefix if timestamps are not disabled in the
/// provided settings.
///
/// # Parameters
/// - `disable_timestamps`: If this value is `true`, the message will be
///   printed without a timestamp prefix. If `false`, the message will be
///   prefixed with a timestamp in the format "[HH:MM:SS] ".
/// - `args`: The arguments to be formatted and printed, following the same
///   syntax as the `println!` macro.
macro_rules! tsprintln {
    ($disable_timestamps:expr, $($args:tt)*) => {{
        if !$disable_timestamps {
            $crate::logging::print_timestamp_prefix();
        }
        println!($($args)*);
    }};
}

/// Prints a timestamped message to standard error if the passed
/// `disable_timestamps` flag is `false`, otherwise just prints the message.
///
/// The message is formatted using the standard `eprintln!` macro, and the
/// timestamp is printed as a prefix if timestamps are not disabled in the
/// provided settings.
///
/// # Parameters
/// - `disable_timestamps`: If this value is `true`, the message will be
///   printed without a timestamp prefix. If `false`, the message will be
///   prefixed with a timestamp in the format "[HH:MM:SS] ".
/// - `args`: The arguments to be formatted and printed, following the same
///   syntax as the `eprintln!` macro.
macro_rules! tseprintln {
    ($disable_timestamps:expr, $($args:tt)*) => {{
        if !$disable_timestamps {
            $crate::logging::eprint_timestamp_prefix();
        }
        eprintln!($($args)*);
    }};
}

// Re-export macros for use in other modules.
pub(crate) use tseprintln;
pub(crate) use tsprintln;
