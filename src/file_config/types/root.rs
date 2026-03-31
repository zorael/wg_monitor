//! Root for the file-based configuration system.

use serde::{Deserialize, Serialize};

use crate::settings;

/// Root configuration structure for the file-based configuration system.
///
/// Its layout is how the settings will be presented in the configuration file.
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct FileConfig {
    /// Monitor settings loaded from the configuration file.
    pub monitor: super::MonitorConfig,

    /// Slack settings loaded from the configuration file.
    pub slack: super::SlackConfig,

    /// Batsign settings loaded from the configuration file.
    pub batsign: super::BatsignConfig,

    /// Command settings loaded from the configuration file.
    pub command: super::CommandConfig,
}

impl From<&settings::Settings> for FileConfig {
    /// Convert from the runtime settings struct to the file configuration struct.
    fn from(s: &settings::Settings) -> Self {
        Self {
            monitor: super::MonitorConfig {
                interface: Some(s.monitor.interface.clone()),
                check_interval: Some(s.monitor.check_interval),
                timeout: Some(s.monitor.timeout),
                reminder_interval: Some(s.monitor.reminder_interval),
                retry_interval: Some(s.monitor.retry_interval),
            },

            slack: super::SlackConfig {
                alert_strings: super::AlertStringsConfig::from(&s.slack.alert_strings),
                reminder_strings: super::ReminderStringsConfig::from(&s.slack.reminder_strings),
                enabled: Some(s.slack.enabled),
                urls: Some(s.slack.urls.clone()),
                show_response: Some(s.slack.show_response),
            },

            batsign: super::BatsignConfig {
                alert_strings: super::AlertStringsConfig::from(&s.batsign.alert_strings),
                reminder_strings: super::ReminderStringsConfig::from(&s.batsign.reminder_strings),
                enabled: Some(s.batsign.enabled),
                urls: Some(s.batsign.urls.clone()),
                show_response: Some(s.batsign.show_response),
            },

            command: super::CommandConfig {
                alert_strings: super::AlertStringsConfig::from(&s.command.alert_strings),
                reminder_strings: super::ReminderStringsConfig::from(&s.command.reminder_strings),
                enabled: Some(s.command.enabled),
                commands: Some(s.command.commands.clone()),
                show_output: Some(s.command.show_output),
            },
        }
    }
}
