//! File configuration structures for the program, which can be deserialized
//! from a configuration file on disk.
//!
//! These structures mirror the runtime settings used by the program,
//! but are designed for deserialization from a file.

use serde::{Deserialize, Serialize};

/// Slack configuration structure. This mirrors the runtime settings struct
/// used by the program for Slack notifications.
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SlackConfig {
    /// Message strings for notifications.
    pub alert_strings: super::AlertStringsConfig,

    /// Message strings for reminder notifications.
    pub reminder_strings: super::ReminderStringsConfig,

    /// Whether Slack notifications are enabled.
    pub enabled: Option<bool>,

    /// The Slack URLs to which the notifications will be sent.
    ///
    /// Each URL is unique to the target Slack channel and includes a token
    /// for authentication.
    pub urls: Option<Vec<String>>,

    /// Whether to print the responses to the HTTP requests to the terminal.
    pub show_response: Option<bool>,
}
