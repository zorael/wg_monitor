//! Defines the types used for runtime settings in the application, including settings for
//! Batsign, Slack, and the base monitoring system.

mod batsign;
mod monitor;
mod pathbufs;
mod settings;
mod slack;
mod strings;

pub use batsign::BatsignSettings;
pub use monitor::MonitorSettings;
pub use pathbufs::PathBufs;
pub use settings::Settings;
pub use slack::SlackSettings;
pub use strings::{MessageStrings, ReminderStrings};
