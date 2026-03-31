//! Settings structures for the program, which hold the runtime settings for various
//! aspects of the program, including notification backends and monitoring settings.
//!
//! These structures are used at runtime to determine how the program behaves,
//! and can be populated from the file-based configuration structures defined in
//! the `file_config` module.
//!
//! Each settings struct has an `apply_file` method that takes the corresponding
//! file configuration struct and applies the settings from the file
//! configuration to the runtime settings, allowing the program to be configured
//! based on the contents of a configuration file on disk.

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
pub use strings::{AlertStrings, ReminderStrings};
