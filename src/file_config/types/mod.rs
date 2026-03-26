//! Configuration structs for the file-based configuration system.
//!
//! Each struct must mirror the corresponding runtime settings struct used by
//! the program for runtime configuration.

mod batsign;
mod command;
mod monitor;
mod root;
mod slack;
mod strings;

pub use batsign::BatsignConfig;
pub use command::CommandConfig;
pub use monitor::MonitorConfig;
pub use root::FileConfig;
pub use slack::SlackConfig;
pub use strings::{MessageStringsConfig, ReminderStringsConfig};
