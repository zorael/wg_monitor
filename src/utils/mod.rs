//! Utility functions and helper methods used across the application,
//! including string manipulation, time formatting, and vector operations.

#![allow(unused_imports)]

mod string;
mod time;
mod vec;

pub use string::{plurality, unescape};
pub use time::{datestamp_now, datestamp_of, fuzzy_datestamp_of, timestamp_now, timestamp_of};
pub use vec::{append_vec_difference, get_elements_not_in_other_vec, trim_vec_of_strings};
