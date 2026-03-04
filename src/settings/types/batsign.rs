//! Batsign notification settings and related functionality.

use crate::file_config;
use crate::settings;
use crate::utils;

/// Batsign runtime settings.
#[derive(Debug)]
pub struct BatsignSettings {
    /// Message strings for Batsign notifications.
    pub strings: settings::MessageStrings,

    /// Message strings for Batsign reminder notifications.
    pub reminder_strings: settings::ReminderStrings,

    /// Whether Batsign notifications are enabled.
    pub enabled: bool,

    /// List of Batsign URLs to send notifications to.
    pub urls: Vec<String>,
}

impl Default for BatsignSettings {
    /// Default values for the Batsign settings.
    fn default() -> Self {
        Self {
            strings: settings::MessageStrings::default(),
            reminder_strings: settings::ReminderStrings::default(),
            enabled: true,
            urls: Vec::new(),
        }
    }
}

impl BatsignSettings {
    /// Applies Batsign settings from the config file, overriding the default
    /// settings where values are available.
    pub fn apply_file(&mut self, batsign_config: &file_config::BatsignConfig) {
        self.strings.apply_file(&batsign_config.strings);
        self.reminder_strings
            .apply_file(&batsign_config.reminder_strings);

        if let Some(enabled) = batsign_config.enabled {
            self.enabled = enabled;
        }

        if let Some(urls) = batsign_config.urls.clone() {
            self.urls = urls;
        }
    }

    /// Trims whitespace from the Batsign URLs and removes any empty ones.
    pub fn trim_urls(&mut self) {
        self.urls = utils::trim_vec_of_strings(&self.urls);
    }

    /// Sanity check the Batsign settings, appending any errors as strings to
    /// the passed vec.
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
