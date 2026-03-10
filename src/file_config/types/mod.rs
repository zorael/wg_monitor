//! Configuration file structures for the program, which can be deserialized
//! from a configuration file on disk.

mod batsign;
mod monitor;
mod root;
mod slack;
mod strings;

pub use batsign::BatsignConfig;
pub use monitor::MonitorConfig;
pub use root::FileConfig;
pub use slack::SlackConfig;
pub use strings::{MessageStringsConfig, ReminderStringsConfig};
