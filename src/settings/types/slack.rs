//! Slack notification settings and related functionality.

use crate::file_config;
use crate::settings;
use crate::utils;

/// Slack settings.
#[derive(Debug)]
pub struct SlackSettings {
    /// Message strings for Slack notifications.
    pub strings: settings::MessageStrings,

    /// Message strings for Slack reminder notifications.
    pub reminder_strings: settings::ReminderStrings,

    /// Whether Slack notifications are enabled.
    pub enabled: bool,

    /// Optional Slack webhook URL for sending notifications to Slack.
    pub urls: Vec<String>,
}

impl Default for SlackSettings {
    /// Default values for the Slack settings.
    fn default() -> Self {
        Self {
            strings: settings::MessageStrings::default(),
            reminder_strings: settings::ReminderStrings::default(),
            enabled: true,
            urls: Vec::new(),
        }
    }
}

impl SlackSettings {
    /// Applies Slack settings from the config file, overriding the default settings where specified.
    pub fn apply_file(&mut self, slack_config: &file_config::SlackConfig) {
        self.strings.apply_file(&slack_config.strings);
        self.reminder_strings
            .apply_file(&slack_config.reminder_strings);

        if let Some(enabled) = slack_config.enabled {
            self.enabled = enabled;
        }

        if let Some(urls) = slack_config.urls.clone() {
            self.urls = urls;
        }
    }

    /// Trims whitespace from the Slack webhook URLs and removes any empty URLs.
    pub fn trim_urls(&mut self) {
        self.urls = utils::trim_vec_of_strings(&self.urls);
    }

    /// Sanity check the Slack settings, returning true if they are valid
    /// and false if any issues are found. This is used to validate the settings
    /// before starting the monitoring loop.
    pub fn sanity_check(&self, vec: &mut Vec<String>) {
        if !self.enabled {
            return;
        }

        if self.urls.is_empty() {
            vec.push(
                "Slack notifications are enabled but no webhook URLs are configured.".to_string(),
            );
            return;
        }

        for url in self.urls.iter() {
            match url.trim() {
                url if !url.starts_with("https://") => {
                    vec.push(format!(
                        "Slack webhook URL \"{url}\" does not seem to be a valid URL."
                    ));
                }
                _ => {}
            }
        }
    }
}
