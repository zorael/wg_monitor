//! Slack configuration structures for the program, which can be deserialized from a
//! configuration file on disk.

use super::*;
use serde::{Deserialize, Serialize};

/// Slack configuration structure. This mirrors the runtime settings struct used
/// by the program for Slack notifications.
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SlackConfig {
    /// Message strings for notifications.
    pub strings: MessageStringsConfig,

    /// Message strings for reminder notifications.
    pub reminder_strings: ReminderStringsConfig,

    /// Whether Slack notifications are enabled.
    pub enabled: Option<bool>,

    /// Optional Slack webhook URL for sending notifications to Slack.
    pub urls: Option<Vec<String>>,
}
