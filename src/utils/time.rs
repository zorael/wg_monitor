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

/// Converts a `time::SystemTime` to a human-readable datestamp string in a
/// fuzzy format that depends on how much time has passed since the given time.
///
/// - If more than 3 days have passed, the datestamp will be in the format
///   "`YYYY-MM-DD`" (not showing the time).
/// - If more than 12 hours but less than 3 days have passed, the datestamp will
///   be in the format "`YYYY-MM-DD HH:MM:SS`" (showing both date and time).
/// - If less than 12 hours have passed, the datestamp will be in the format
///   "`HH:MM:SS`" (showing only the time).
///
/// # Parameters
/// - `when`: A reference to a `time::SystemTime` instance representing the time
///   to be converted to a fuzzy datestamp string.
///
/// # Returns
/// A `String` containing the formatted fuzzy datestamp based on how much time has
/// passed since the given time, in one of the formats described above.
pub fn fuzzy_datestamp_of(when: &time::SystemTime) -> String {
    const THREE_DAYS: time::Duration = time::Duration::from_secs(3 * 24 * 3600);
    const TWELVE_HOURS: time::Duration = time::Duration::from_secs(12 * 3600);

    let now = time::SystemTime::now();
    let since = now.duration_since(*when).unwrap_or_default();
    let datetime: chrono::DateTime<chrono::Local> = (*when).into();

    if since > THREE_DAYS {
        datetime.format("%Y-%m-%d").to_string()
    } else if since > TWELVE_HOURS {
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        datetime.format("%H:%M:%S").to_string()
    }
}
