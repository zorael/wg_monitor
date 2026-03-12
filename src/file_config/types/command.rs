//! Configuration for the Command backend.

use super::*;
use serde::{Deserialize, Serialize};

/// Command configuration structure. This mirrors the runtime settings struct used
/// by the program for Command notifications.
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CommandConfig {
    /// Message strings for notifications.
    pub strings: MessageStringsConfig,

    /// Message strings for reminder notifications.
    pub reminder_strings: ReminderStringsConfig,

    /// Whether Command notifications are enabled.
    pub enabled: Option<bool>,

    /// The commands to execute for notifications.
    pub commands: Option<Vec<String>>,
}
