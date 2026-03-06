//! Time-related utility functions.

#![allow(dead_code)]

use std::time;

/// Converts a `SystemTime` to a human-readable timestamp string in the format "HH:MM:SS".
pub fn timestamp_of(when: &time::SystemTime) -> String {
    let datetime: chrono::DateTime<chrono::Local> = (*when).into();
    datetime.format("%H:%M:%S").to_string()
}

/// Converts a `SystemTime` to a human-readable datestamp string in the format "YYYY-MM-DD HH:MM:SS".
pub fn datestamp_of(when: &time::SystemTime) -> String {
    let datetime: chrono::DateTime<chrono::Local> = (*when).into();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Returns the current local time as a timestamp string in the format "HH:MM:SS".
pub fn timestamp_now() -> String {
    let datetime: chrono::DateTime<chrono::Local> = time::SystemTime::now().into();
    datetime.format("%H:%M:%S").to_string()
}

/// Returns the current local time as a datestamp string in the format "YYYY-MM-DD HH:MM:SS".
pub fn datestamp_now() -> String {
    let datetime: chrono::DateTime<chrono::Local> = time::SystemTime::now().into();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}
