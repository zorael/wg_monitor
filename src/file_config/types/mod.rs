//! Configuration structures for the file-based configuration system.

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
