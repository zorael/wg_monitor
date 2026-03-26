//! Configuration structures for the Batsign notification backend.

use super::*;
use serde::{Deserialize, Serialize};

/// Batsign configuration structure. This mirrors the runtime settings struct
/// used by the program for Batsign notifications.
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct BatsignConfig {
    /// Message strings for notifications.
    pub strings: MessageStringsConfig,

    /// Message strings for reminder notifications.
    pub reminder_strings: ReminderStringsConfig,

    /// Whether Batsign notifications are enabled.
    pub enabled: Option<bool>,

    /// The Batsign URLs to which the notifications will be sent.
    ///
    /// Each URL is unique to the target email address and includes a token
    /// for authentication.
    pub urls: Option<Vec<String>>,
}
