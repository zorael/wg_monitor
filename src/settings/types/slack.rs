//! Types and implementation for Slack runtime settings.

use crate::file_config;
use crate::settings;
use crate::utils;

/// Runtime settings for the Slack notification backend.
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

    /// Whether to print the response to the HTTP request to the terminal.
    pub show_response: bool,
}

impl SlackSettings {
    /// Applies settings from a `file_config::SlackConfig` to the current
    /// `SlackSettings` instance.
    ///
    /// # Parameters
    /// - `slack_config`: The `file_config::SlackConfig`
    ///   containing the settings to apply to the current `SlackSettings` instance.
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

        if let Some(show_response) = slack_config.show_response {
            self.show_response = show_response;
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
