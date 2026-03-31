//! Settings types for the program, with message strings for notifications and
//! reminders.
//!
//! This module contains the `MessageStrings` and `ReminderStrings` structs,
//! which hold the message string settings for notifications and reminders,
//! respectively.
//!
//! These structs include fields for various message components, such as headers,
//! footers, and bullet points, as well as message templates for peers with and
//! without timestamps. The module also includes methods for applying settings
//! from configuration file structures to the runtime settings structs, allowing
//! for easy updates of the message strings based on user-provided configuration
//! files.
//!
//! Each notification backend (e.g., Slack, Batsign, Command) has their own instance of
//! these settings structs that include these message strings as fields. It is through
//! these that message strings are formatted.

use crate::file_config;

/// Message string settings struct for the program.
///
/// This *must* mirror `file_config::MessageStringsConfig`.
#[derive(Clone, Debug)]
pub struct AlertStrings {
    /// Message header.
    pub header: String,

    /// Message header for the first run (first loop) of the program.
    ///
    /// This is used instead of `header` at such times.
    pub first_run_header: String,

    /// Section header for peers that are missing on the first run of the program.
    pub first_run_missing: String,

    /// Section header for peers that were lost (timed out) since the last check.
    pub lost: String,

    /// Section header for peers that were present but are now missing, usually
    /// (always?) due to a restart of the VPN.
    pub forgot: String,

    /// Section header for peers that appeared for the first time since the last check.
    pub appeared: String,

    /// Section header for peers that returned after being lost (timed out).
    pub returned: String,

    /// Section header for peers that are still lost (timed out).
    pub still_lost: String,

    /// Section header for peers that have still not appeared since the last restart.
    pub still_missing: String,

    /// Message footer.
    pub footer: String,

    /// Bullet point string for listing peers.
    pub bullet_point: String,

    /// Message string for a peer with a timestamp, used in notifications
    /// when the timestamp of the last seen time is known.
    ///
    /// This translates to "lost" peers.
    pub peer_with_timestamp: String,

    /// Message string for a peer without a timestamp, used in notifications
    /// when the timestamp of the last seen time is zero.
    ///
    /// This translates to "missing" peers, peers that just returned and
    /// peers that just appeared, since in all such cases the delta of the
    /// peer's last seen time is less than or equal to the check interval.
    pub peer_no_timestamp: String,

    /// Message string for a peer that is returning with a timestamp.
    ///
    /// This translates to "returning" and "appearing" peers.
    pub returning_peer_with_timestamp: String,
}

impl Default for AlertStrings {
    /// Default values for the message strings, used as a base for applying
    /// configuration file overrides.
    fn default() -> Self {
        Self {
            header: "WireGuard Monitor report\\n".to_string(),
            first_run_header: "WireGuard Monitor starting up\\n".to_string(),
            first_run_missing: "Missing:\\n".to_string(),
            lost: "Lost:\\n".to_string(),
            forgot: "Lost to a network reset:\\n".to_string(),
            appeared: "Just appeared:\\n".to_string(),
            returned: "Returned:\\n".to_string(),
            still_lost: "Still lost:\\n".to_string(),
            still_missing: "Still have yet to see (since last restart):\\n".to_string(),
            footer: String::new(),
            bullet_point: " - ".to_string(),
            peer_with_timestamp: "{peer} (last seen {when})".to_string(),
            peer_no_timestamp: "{peer}".to_string(),
            returning_peer_with_timestamp: "{peer} (returned {when})".to_string(),
        }
    }
}

impl AlertStrings {
    /// Applies the values from a `file_config::MessageStringsConfig` to the
    /// current `MessageStrings` instance.
    ///
    /// This allows for overriding the default message strings with values from
    /// the configuration file, while still falling back to defaults for any
    /// values that are not provided in it.
    ///
    /// # Parameters
    /// - `config`: The `file_config::MessageStringsConfig` containing the
    ///   message string settings to apply to the current `MessageStrings` instance.
    pub fn apply_file(&mut self, config: &file_config::AlertStringsConfig) {
        if let Some(header) = &config.header {
            self.header = header.clone();
        }

        if let Some(first_run_header) = &config.first_run_header {
            self.first_run_header = first_run_header.clone();
        }

        if let Some(first_run_missing) = &config.first_run_missing {
            self.first_run_missing = first_run_missing.clone();
        }

        if let Some(lost) = &config.lost {
            self.lost = lost.clone();
        }

        if let Some(forgot) = &config.forgot {
            self.forgot = forgot.clone();
        }

        if let Some(appeared) = &config.appeared {
            self.appeared = appeared.clone();
        }

        if let Some(returned) = &config.returned {
            self.returned = returned.clone();
        }

        if let Some(still_lost) = &config.still_lost {
            self.still_lost = still_lost.clone();
        }

        if let Some(still_missing) = &config.still_missing {
            self.still_missing = still_missing.clone();
        }

        if let Some(footer) = &config.footer {
            self.footer = footer.clone();
        }

        if let Some(bullet_point) = &config.bullet_point {
            self.bullet_point = bullet_point.clone();
        }

        if let Some(peer_with_timestamp) = &config.peer_with_timestamp {
            self.peer_with_timestamp = peer_with_timestamp.clone();
        }

        if let Some(peer_no_timestamp) = &config.peer_no_timestamp {
            self.peer_no_timestamp = peer_no_timestamp.clone();
        }

        if let Some(returning_peer_with_timestamp) = &config.returning_peer_with_timestamp {
            self.returning_peer_with_timestamp = returning_peer_with_timestamp.clone();
        }
    }
}

/// Reminder message string settings struct for the program.
/// This must mirror `file_config::ReminderStringsConfig`.
#[derive(Clone, Debug)]
pub struct ReminderStrings {
    /// Message header for reminder notifications.
    pub header: String,

    /// Section header for peers that are still lost (timed out).
    pub still_lost: String,

    /// Section header for peers that have still not appeared since the last restart.
    pub still_missing: String,

    /// Message footer.
    pub footer: String,

    /// Bullet point string for listing peers.
    pub bullet_point: String,

    /// Message string for a peer with a timestamp, used in notifications
    /// when the timestamp of the last seen time is known.
    ///
    /// This translates to "lost" peers.
    pub peer_with_timestamp: String,

    /// Message string for a peer without a timestamp, used in notifications
    /// when the timestamp of the last seen time is zero.
    ///
    /// This translates to "missing" peers.
    pub peer_no_timestamp: String,
}

impl Default for ReminderStrings {
    /// Default values for the reminder message strings, used as a base for
    /// applying configuration file overrides.
    fn default() -> Self {
        Self {
            header: "WireGuard Monitor reminder\\n".to_string(),
            still_lost: "Still lost:\\n".to_string(),
            still_missing: "Still have yet to see (since last restart):\\n".to_string(),
            footer: String::new(),
            bullet_point: " - ".to_string(),
            peer_with_timestamp: "{peer} (last seen {when})".to_string(),
            peer_no_timestamp: "{peer}".to_string(),
        }
    }
}

impl ReminderStrings {
    /// Applies the values from a `file_config::ReminderStringsConfig` to the
    /// current `ReminderStrings` instance.
    ///
    /// This allows for overriding the default reminder message strings with
    /// values from the configuration file, while still falling back to defaults
    /// for any values that are not provided in it.
    ///
    /// # Parameters
    /// - `config`: The `file_config::ReminderStringsConfig` containing the
    ///   reminder string settings to apply to the current `ReminderStrings` instance.
    pub fn apply_file(&mut self, config: &file_config::ReminderStringsConfig) {
        if let Some(header) = &config.header {
            self.header = header.clone();
        }

        if let Some(still_lost) = &config.still_lost {
            self.still_lost = still_lost.clone();
        }

        if let Some(still_missing) = &config.still_missing {
            self.still_missing = still_missing.clone();
        }

        if let Some(footer) = &config.footer {
            self.footer = footer.clone();
        }

        if let Some(bullet_point) = &config.bullet_point {
            self.bullet_point = bullet_point.clone();
        }

        if let Some(peer_with_timestamp) = &config.peer_with_timestamp {
            self.peer_with_timestamp = peer_with_timestamp.clone();
        }

        if let Some(peer_no_timestamp) = &config.peer_no_timestamp {
            self.peer_no_timestamp = peer_no_timestamp.clone();
        }
    }
}
