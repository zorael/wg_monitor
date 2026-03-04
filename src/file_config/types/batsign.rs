//! Batsign backend configuration structures for the program, which can be deserialized from a
//! configuration file on disk.

use super::*;
use serde::{Deserialize, Serialize};

/// Batsign configuration structure. This mirrors the runtime settings struct
/// used by the program for the Batsign backend.
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct BatsignConfig {
    /// Message strings for notifications.
    pub strings: MessageStringsConfig,

    /// Message strings for reminder notifications.
    pub reminder_strings: ReminderStringsConfig,

    /// Whether Batsign notifications are enabled.
    pub enabled: Option<bool>,

    /// List of URLs to send Batsign notifications to.
    pub urls: Option<Vec<String>>,
}
