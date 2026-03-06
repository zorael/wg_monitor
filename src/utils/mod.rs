//! Miscellaneous utility functions.

#![allow(unused_imports)]

mod time;
mod vec;

pub use time::{datestamp_now, datestamp_of, timestamp_now, timestamp_of};
pub use vec::{append_vec_difference, trim_vec_of_strings};
pub use vec::{get_elements_not_in_other_vec, get_vec_difference};
