//! Defines the types used for runtime settings in the application, including settings for
//! Batsign, Slack, and the base monitoring system.

mod batsign;
mod command;
mod monitor;
mod pathbufs;
mod root;
mod slack;
mod strings;

pub use batsign::BatsignSettings;
pub use command::CommandSettings;
pub use monitor::MonitorSettings;
pub use pathbufs::PathBufs;
pub use root::Settings;
pub use slack::SlackSettings;
pub use strings::{MessageStrings, ReminderStrings};
