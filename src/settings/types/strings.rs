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
//! Each notification backend (e.g., Slack, Batsign, Command) has their own
//! settings structs that include these message strings as fields. It is through
//! these that message strings can be formatted.

use crate::file_config;

/// Message string settings struct for the program.
/// This must mirror `file_config::MessageStringsConfig`.
#[derive(Clone, Debug)]
pub struct MessageStrings {
    /// Message header for notifications.
    ///
    /// This is the main header that appears at the top of the notification message.
    /// Depending on the backend, it may be used as subject line.
    pub header: String,

    /// This is the main header that appears at the top of the notification
    /// message on the first run of the program, when there is no previous state
    /// to compare against. This can be used to indicate that the program is
    /// starting up and provide an initial report of the current state of peers.
    ///
    /// It replaces `header` for the first run of the program, and is only used
    /// at such times.
    ///
    /// Depending on the backend, it may be used as subject line.
    pub first_run_header: String,

    /// Section header for peers that are missing on the first run of the program.
    ///
    /// This is used in the initial report of the current state of peers when the
    /// program is run for the first time, and indicates which peers are currently
    /// missing (i.e., have never been seen before).
    pub first_run_missing: String,

    /// Section header for peers that were present but are now missing, usually
    /// due to a timeout.
    ///
    /// This indicates which peers were seen in the previous check but are now
    /// missing, which means their handshake has timed out.
    pub lost: String,

    /// Section header for peers that were present but are now missing, usually
    /// due to a restart of the VPN.
    ///
    /// This indicates which peers were seen in the previous check but are now
    /// missing, which should not happen if the VPN is running continuously.
    pub forgot: String,

    /// Section header for peers that appeared for the first time since the last check.
    ///
    /// This indicates peers which had never been seen since the last VPN restart
    /// but were now seen for the first time.
    pub appeared: String,

    /// Section header for peers that returned after being lost (timed out).
    ///
    /// This indicates peers whose handshake previously timed out (making them
    /// "lost"), but have now been seen again, meaning they have returned.
    pub returned: String,

    /// Section header for peers that are still lost (timed out).
    ///
    /// This indicates peers whose handshake has timed out and have still not
    /// been seen since then.
    pub still_lost: String,

    /// Section header for peers that have still not appeared since the last restart.
    pub still_missing: String,

    /// Message footer for notifications.
    ///
    /// This appears at the end of the notification message, and can be used
    /// for any additional information or static closing remarks.
    pub footer: String,

    /// Bullet point string for listing peers in notifications.
    pub bullet_point: String,

    /// Message string for a peer with a timestamp, used in notifications
    /// when the timestamp of the last seen time is known.
    ///
    /// This translates to "lost" peers.
    pub peer_with_timestamp: String,

    /// Message string for a peer without a timestamp, used in notifications
    /// when the timestamp is either unknown (peer is "missing") or when that
    /// time is right now (the peer just "appeared" or "returned").
    pub peer_no_timestamp: String,
}

impl Default for MessageStrings {
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
        }
    }
}

impl MessageStrings {
    /// Applies the values from a `file_config::MessageStringsConfig` to the
    /// current `MessageStrings` instance.
    ///
    /// This allows for overriding the default message strings with values from
    /// the configuration file, while still falling back to defaults for any
    /// values that are not provided in the configuration file.
    ///
    /// # Parameters
    /// - `config`: The `file_config::MessageStringsConfig` containing the
    ///   message string settings to apply to the current `MessageStrings` instance.
    pub fn apply_file(&mut self, config: &file_config::MessageStringsConfig) {
        if let Some(header) = config.header.clone() {
            self.header = header;
        }

        if let Some(first_run_header) = config.first_run_header.clone() {
            self.first_run_header = first_run_header;
        }

        if let Some(first_run_missing) = config.first_run_missing.clone() {
            self.first_run_missing = first_run_missing;
        }

        if let Some(lost) = config.lost.clone() {
            self.lost = lost;
        }

        if let Some(forgot) = config.forgot.clone() {
            self.forgot = forgot;
        }

        if let Some(appeared) = config.appeared.clone() {
            self.appeared = appeared;
        }

        if let Some(returned) = config.returned.clone() {
            self.returned = returned;
        }

        if let Some(still_lost) = config.still_lost.clone() {
            self.still_lost = still_lost;
        }

        if let Some(still_missing) = config.still_missing.clone() {
            self.still_missing = still_missing;
        }

        if let Some(footer) = config.footer.clone() {
            self.footer = footer;
        }

        if let Some(bullet_point) = config.bullet_point.clone() {
            self.bullet_point = bullet_point;
        }

        if let Some(peer_with_timestamp) = config.peer_with_timestamp.clone() {
            self.peer_with_timestamp = peer_with_timestamp;
        }

        if let Some(peer_no_timestamp) = config.peer_no_timestamp.clone() {
            self.peer_no_timestamp = peer_no_timestamp;
        }
    }
}

/// Reminder message string settings struct for the program.
/// This must mirror `file_config::ReminderStringsConfig`.
#[derive(Clone, Debug)]
pub struct ReminderStrings {
    /// Message header for reminders.
    ///
    /// This is the main header that appears at the top of the reminder message.
    /// Depending on the backend, it may be used as subject line.
    pub header: String,

    /// Section header for peers that are still lost (timed out).
    ///
    /// This indicates peers whose handshake has timed out and have still not
    /// been seen since then.
    pub still_lost: String,

    /// Section header for peers that have still not appeared since the last restart.
    pub still_missing: String,

    /// Message footer for reminders.
    ///
    /// This appears at the end of the reminder message, and can be used
    /// for any additional information or static closing remarks.
    pub footer: String,

    /// Bullet point string for listing peers in reminders.
    pub bullet_point: String,

    /// Message string for a peer with a timestamp, used in reminders
    /// when the timestamp of the last seen time is known.
    ///
    /// This translates to "lost" peers.
    pub peer_with_timestamp: String,

    /// Message string for a peer without a timestamp, used in reminders
    /// when the timestamp is unknown (peer is "missing").
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
    /// for any values that are not provided in the configuration file.
    ///
    /// # Parameters
    /// - `config`: The `file_config::ReminderStringsConfig` containing the
    ///   reminder string settings to apply to the current `ReminderStrings` instance.
    pub fn apply_file(&mut self, config: &file_config::ReminderStringsConfig) {
        if let Some(header) = config.header.clone() {
            self.header = header;
        }

        if let Some(still_lost) = config.still_lost.clone() {
            self.still_lost = still_lost;
        }

        if let Some(still_missing) = config.still_missing.clone() {
            self.still_missing = still_missing;
        }

        if let Some(footer) = config.footer.clone() {
            self.footer = footer;
        }

        if let Some(bullet_point) = config.bullet_point.clone() {
            self.bullet_point = bullet_point;
        }

        if let Some(peer_with_timestamp) = config.peer_with_timestamp.clone() {
            self.peer_with_timestamp = peer_with_timestamp;
        }

        if let Some(peer_no_timestamp) = config.peer_no_timestamp.clone() {
            self.peer_no_timestamp = peer_no_timestamp;
        }
    }
}
