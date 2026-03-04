//! FIXME

mod batsign;
mod fileconfig;
mod monitor;
mod slack;
mod strings;

pub use batsign::BatsignConfig;
pub use fileconfig::FileConfig;
pub use monitor::MonitorConfig;
pub use slack::SlackConfig;
pub use strings::{MessageStringsConfig, ReminderStringsConfig};
