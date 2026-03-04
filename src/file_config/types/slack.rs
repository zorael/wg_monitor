//! FIXME

use super::*;
use serde::{Deserialize, Serialize};

/// Slack configuration structures for the program, which can be deserialized from a
/// configuration file on disk. These structures mirror the settings used by the program.
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
