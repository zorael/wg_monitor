//! This module defines the structure of the configuration file used by the
//! Wireguard monitor program, as well as functions for deserializing the
//! configuration from disk and resolving the default configuration directory
//! from environment variables.
//!
//! The configuration file allows users to override default settings for
//! monitoring and notifications, and is loaded at runtime to determine how
//! the program should operate.

mod io;
mod paths;
mod types;

pub use io::deserialize_config_file;
pub use paths::resolve_default_config_directory_from_env;
pub use types::{BatsignConfig, FileConfig, MonitorConfig, SlackConfig};
pub use types::{MessageStringsConfig, ReminderStringsConfig};
