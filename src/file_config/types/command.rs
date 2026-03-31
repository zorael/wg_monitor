//! Configuration structures for the Command notification backend.

use serde::{Deserialize, Serialize};

/// Command configuration structure. This mirrors the runtime settings struct
/// used by the program for Command notifications.
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CommandConfig {
    /// Message strings for alert notifications.
    pub alert_strings: super::AlertStringsConfig,

    /// Message strings for reminder notifications.
    pub reminder_strings: super::ReminderStringsConfig,

    /// Whether Command notifications are enabled.
    pub enabled: Option<bool>,

    /// The command strings to execute for notifications.
    ///
    /// Each command string is executed as a separate process.
    pub commands: Option<Vec<String>>,

    /// Whether to print the output of the executed commands to the terminal.
    pub show_output: Option<bool>,
}
