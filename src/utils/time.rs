//! Utility functions for time formatting and conversion.

#![allow(dead_code)]

use std::time;

/// Converts a `time::SystemTime` to a human-readable timestamp string in the
/// format "`HH:MM:SS`".
///
/// # Parameters
/// - `when`: A reference to a `time::SystemTime` instance representing the time
///   to be converted to a timestamp string.
///
/// # Returns
/// A `String` containing the formatted timestamp in "`HH:MM:SS`" format.
pub fn timestamp_of(when: &time::SystemTime) -> String {
    let datetime: chrono::DateTime<chrono::Local> = (*when).into();
    datetime.format("%H:%M:%S").to_string()
}

/// Converts a `time::SystemTime` to a human-readable datestamp string in the
/// format "`YYYY-MM-DD HH:MM:SS`".
///
/// # Parameters
/// - `when`: A reference to a `time::SystemTime` instance representing the time
///   to be converted to a datestamp string.
///
/// # Returns
/// A `String` containing the formatted datestamp in "`YYYY-MM-DD HH:MM:SS`" format.
pub fn datestamp_of(when: &time::SystemTime) -> String {
    let datetime: chrono::DateTime<chrono::Local> = (*when).into();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Returns the current local time as a timestamp string in the format "`HH:MM:SS`".
///
/// The timestamp is generated based on the current system time and formatted
/// using the local timezone.
///
/// # Returns
/// A `String` containing the current time formatted as a timestamp in "`HH:MM:SS`" format.
pub fn timestamp_now() -> String {
    timestamp_of(&time::SystemTime::now())
}

/// Returns the current local time as a datestamp string in the format
/// "`YYYY-MM-DD HH:MM:SS`".
///
/// The datestamp is generated based on the current system time and formatted
/// using the local timezone.
///
/// # Returns
/// A `String` containing the current time formatted as a datestamp in
/// "`YYYY-MM-DD HH:MM:SS`" format.
pub fn datestamp_now() -> String {
    datestamp_of(&time::SystemTime::now())
}
