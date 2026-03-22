//! Utility functions and helper methods used across the application,
//! including string manipulation, time formatting, and vector operations.

#![allow(unused_imports)]

mod string;
mod time;
mod vec;

pub use string::{escape_json, unescape};
pub use time::{datestamp_now, datestamp_of, timestamp_now, timestamp_of};
pub use vec::{append_vec_difference, trim_vec_of_strings};
pub use vec::{get_elements_not_in_other_vec, get_vec_difference};
