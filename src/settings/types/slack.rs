//! Types and implementation for Slack runtime settings, which include message
//! strings, enabled status, and notification URLs.
//!
//! This module defines the `SlackSettings` struct, which holds the runtime
//! settings for the Slack notification backend, including message strings for
//! notifications and reminders, whether Slack notifications are enabled, and
//! the list of Slack webhook URLs to which notifications will be sent.

use crate::file_config;
use crate::settings;
use crate::utils;

/// Runtime settings for the Slack notification backend, including message
/// strings, enabled status, and notification URLs.
#[derive(Debug, Default)]
pub struct SlackSettings {
    /// Message strings for Slack notifications.
    pub strings: settings::MessageStrings,

    /// Message strings for Slack reminder notifications.
    pub reminder_strings: settings::ReminderStrings,

    /// Whether Slack notifications are enabled.
    pub enabled: bool,

    /// The Slack webhook URLs to which the notifications will be sent.
    ///
    /// Each URL is unique to the target Slack channel and includes a token
    /// for authentication.
    pub urls: Vec<String>,
}

impl SlackSettings {
    /// Applies settings from a `file_config::SlackConfig` to the current
    /// `SlackSettings` instance, updating the message strings, enabled status,
    /// and notification URLs based on the values provided in the file configuration.
    ///
    /// # Parameters
    /// - `slack_config`: The `file_config::SlackConfig`
    ///   containing the settings to apply to the current `SlackSettings`
    ///   instance. This includes message strings for notifications and reminders,
    ///   the enabled status, and the list of Slack webhook URLs.
    pub fn apply_file(&mut self, slack_config: &file_config::SlackConfig) {
        self.strings.apply_file(&slack_config.strings);
        self.reminder_strings
            .apply_file(&slack_config.reminder_strings);

        if let Some(enabled) = slack_config.enabled {
            self.enabled = enabled;
        }

        if let Some(urls) = &slack_config.urls {
            self.urls = urls.clone();
        }
    }

    /// Trims whitespace from the Slack webhook URLs in the settings, which can
    /// help to avoid issues with URLs that have leading or trailing whitespace
    /// that could cause problems when sending notifications.
    pub fn trim_urls(&mut self) {
        self.urls = utils::trim_vec_of_strings(&self.urls);
    }

    /// Performs a sanity check on the Slack settings, validating that if
    /// Slack notifications are enabled, there are at the same time webhook URLs
    /// configured, and that the URLs appear to be valid.
    ///
    /// If any issues are found, descriptive error messages are added to the
    /// provided vector of strings.
    ///
    /// # Parameters
    /// - `vec`: A mutable reference to a vector of strings to which any error
    ///   messages will be added if issues are found with the Slack settings.
    ///   If the settings are valid, this vector will remain unchanged.
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
