//! Types and implementations for Batsign runtime settings.

use crate::file_config;
use crate::settings;
use crate::utils;

/// Runtime settings for the Batsign notification backend, including message
/// strings, enabled status, and notification URLs.
#[derive(Debug, Default)]
pub struct BatsignSettings {
    /// Message strings for Batsign notifications.
    pub strings: settings::MessageStrings,

    /// Message strings for Batsign reminder notifications.
    pub reminder_strings: settings::ReminderStrings,

    /// Whether Batsign notifications are enabled.
    pub enabled: bool,

    /// The Batsign URLs to which the notifications will be sent.
    ///
    /// Each URL is unique to the target email address and includes a token
    /// for authentication.
    pub urls: Vec<String>,
}

impl BatsignSettings {
    /// Applies settings from a `file_config::BatsignConfig` to the current
    /// `BatsignSettings` instance.
    ///
    /// # Parameters
    /// - `batsign_config`: The `file_config::BatsignConfig`
    ///   containing the settings to apply to the current `BatsignSettings`
    ///   instance.
    pub fn apply_file(&mut self, batsign_config: &file_config::BatsignConfig) {
        self.strings.apply_file(&batsign_config.strings);
        self.reminder_strings
            .apply_file(&batsign_config.reminder_strings);

        if let Some(enabled) = batsign_config.enabled {
            self.enabled = enabled;
        }

        if let Some(urls) = &batsign_config.urls {
            self.urls = urls.clone();
        }
    }

    /// Trims whitespace from the Batsign URLs in the settings, which can help
    /// to avoid issues with URLs that have leading or trailing whitespace that
    /// could cause problems when sending notifications.
    pub fn trim_urls(&mut self) {
        self.urls = utils::trim_vec_of_strings(&self.urls);
    }

    /// Performs a sanity check on the Batsign settings, validating that if
    /// Batsign notifications are enabled, there are at the same time URLs
    /// configured, and that the URLs appear to be valid.
    ///
    /// If any issues are found, descriptive error messages are added to the
    /// provided vector of strings.
    ///
    /// # Parameters
    /// - `vec`: A mutable reference to a vector of strings to which any error
    ///   messages will be added if issues are found with the Batsign settings.
    ///   If the settings are valid, this vector will remain unchanged.
    pub fn sanity_check(&self, vec: &mut Vec<String>) {
        if !self.enabled {
            return;
        }

        if self.urls.is_empty() {
            vec.push("Batsign notifications are enabled but no URLs are configured.".to_string());
            return;
        }

        for url in self.urls.iter() {
            match url.trim() {
                url if !url.starts_with("https://") => {
                    vec.push(format!(
                        "Batsign URL \"{url}\" does not seem to be a valid URL."
                    ));
                }
                _ => {}
            }
        }
    }
}
