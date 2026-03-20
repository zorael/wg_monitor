//! Root configuration structure for the file-based configuration system.

use super::*;
use serde::{Deserialize, Serialize};

use crate::settings;

/// Root configuration structure for the file-based configuration system.
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct FileConfig {
    /// Monitor settings loaded from the configuration file.
    pub monitor: MonitorConfig,

    /// Slack settings loaded from the configuration file.
    pub slack: SlackConfig,

    /// Batsign settings loaded from the configuration file.
    pub batsign: BatsignConfig,

    /// Command settings loaded from the configuration file.
    pub command: CommandConfig,
}

impl From<&settings::Settings> for FileConfig {
    /// Convert from the runtime settings struct to the file configuration struct.
    fn from(s: &settings::Settings) -> Self {
        Self {
            monitor: MonitorConfig {
                interface: Some(s.monitor.interface.clone()),
                check_interval: Some(s.monitor.check_interval),
                timeout: Some(s.monitor.timeout),
                reminder_interval: Some(s.monitor.reminder_interval),
                retry_interval: Some(s.monitor.retry_interval),
            },

            slack: SlackConfig {
                strings: MessageStringsConfig::from(&s.slack.strings),
                reminder_strings: ReminderStringsConfig::from(&s.slack.reminder_strings),
                enabled: Some(s.slack.enabled),
                urls: Some(s.slack.urls.clone()),
            },

            batsign: BatsignConfig {
                strings: MessageStringsConfig::from(&s.batsign.strings),
                reminder_strings: ReminderStringsConfig::from(&s.batsign.reminder_strings),
                enabled: Some(s.batsign.enabled),
                urls: Some(s.batsign.urls.clone()),
            },

            command: CommandConfig {
                strings: MessageStringsConfig::from(&s.command.strings),
                reminder_strings: ReminderStringsConfig::from(&s.command.reminder_strings),
                enabled: Some(s.command.enabled),
                commands: Some(s.command.commands.clone()),
            },
        }
    }
}
