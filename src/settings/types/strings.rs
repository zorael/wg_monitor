//! This module houses structs and functions related to the message strings
//! used in notifications and reminders for the Wireguard monitor program.

use crate::file_config;

/// Notification message string settings struct for the program.
/// This must mirror `file_config::MessageStringsConfig`.
#[derive(Clone, Debug)]
pub struct MessageStrings {
    /// Message header.
    pub header: String,

    /// Message header for the first run (first loop) of the program. This is
    /// used instead of `header` at such times.
    pub first_run_header: String,

    /// Section header for peers that are missing on the first run of the program.
    pub first_run_missing: String,

    /// Section header for peers that were lost (timed out) since the last check.
    pub lost: String,

    /// Section header for peers that were present but are now missing, usually
    /// (or always?) due to a restart of the VPN.
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
    /// when the timestamp of the last seen time is known. This translates to
    /// "lost" peers.
    pub peer_with_timestamp: String,

    /// Message string for a peer without a timestamp, used in notifications
    /// when the timestamp of the last seen time is zero. This translates to
    /// "missing" peers.
    pub peer_no_timestamp: String,
}

impl Default for MessageStrings {
    /// Default values for the message strings, used as a base for applying
    /// config file overrides.
    fn default() -> Self {
        Self {
            header: "Wireguard Monitor report\\n".to_string(),
            first_run_header: "Wireguard Monitor starting up\\n".to_string(),
            first_run_missing: "Missing:\\n".to_string(),
            lost: "Lost:\\n".to_string(),
            forgot: "Lost track of due to a restart of the VPN:\\n".to_string(),
            appeared: "Just appeared:\\n".to_string(),
            returned: "Returned:\\n".to_string(),
            still_lost: "Still lost:\\n".to_string(),
            still_missing: "Still have yet to see (since last restart):\\n".to_string(),
            footer: String::new(),
            bullet_point: "- ".to_string(),
            peer_with_timestamp: "{peer} (last seen {when})".to_string(),
            peer_no_timestamp: "{peer}".to_string(),
        }
    }
}

impl MessageStrings {
    /// Applies the values from a `file_config::strings::MessageStringsConfig`
    /// to the current `MessageStrings` instance.
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
    /// Message header.
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
    /// when the timestamp of the last seen time is known. This translates to
    /// "lost" peers.
    pub peer_with_timestamp: String,

    /// Message string for a peer without a timestamp, used in notifications
    /// when the timestamp of the last seen time is zero. This translates to
    /// "missing" peers.
    pub peer_no_timestamp: String,
}

impl Default for ReminderStrings {
    /// Default values for the reminder message strings, used as a base for
    /// applying configuration file overrides.
    fn default() -> Self {
        Self {
            header: "Wireguard Monitor reminder\\n".to_string(),
            still_lost: "Still lost:\\n".to_string(),
            still_missing: "Still have yet to see (since last restart):\\n".to_string(),
            footer: String::new(),
            bullet_point: "- ".to_string(),
            peer_with_timestamp: "{peer} (last seen {when})".to_string(),
            peer_no_timestamp: "{peer}".to_string(),
        }
    }
}

impl ReminderStrings {
    /// Applies the values from a `file_config::strings::ReminderStringsConfig`
    /// to the current `ReminderStrings` instance.
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
