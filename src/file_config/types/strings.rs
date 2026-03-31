//! String structs for the file-based configuration system.

use serde::{Deserialize, Serialize};

use crate::settings;

/// Alert string configuration struct.
///
/// This mirrors the runtime settings struct used by the program for alert strings.
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AlertStringsConfig {
    /// Message header.
    pub header: Option<String>,

    /// Message header for the first run (first loop) of the program.
    ///
    /// This is used instead of `header` at such times.
    pub first_run_header: Option<String>,

    /// Section header for peers that are missing on the first run of the program.
    pub first_run_missing: Option<String>,

    /// Section header for peers that were lost (timed out) since the last check.
    pub lost: Option<String>,

    /// Section header for peers that were present but are now missing, usually
    /// (always?) due to a restart of the VPN.
    pub forgot: Option<String>,

    /// Section header for peers that appeared for the first time since the last check.
    pub appeared: Option<String>,

    /// Section header for peers that returned after being lost (timed out).
    pub returned: Option<String>,

    /// Section header for peers that are still lost (timed out).
    pub still_lost: Option<String>,

    /// Section header for peers that have still not appeared since the last restart.
    pub still_missing: Option<String>,

    /// Message footer.
    pub footer: Option<String>,

    /// Bullet point string for listing peers.
    pub bullet_point: Option<String>,

    /// Message string for a peer with a timestamp, used in notifications
    /// when the timestamp of the last seen time is known.
    ///
    /// This translates to "lost" peers.
    pub peer_with_timestamp: Option<String>,

    /// Message string for a peer without a timestamp, used in notifications
    /// when the timestamp of the last seen time is zero.
    ///
    /// This translates to "missing" peers, peers that just returned and
    /// peers that just appeared, since in all such cases the delta of the
    /// peer's last seen time is less than or equal to the check interval.
    pub peer_no_timestamp: Option<String>,

    /// Message string for a peer that is returning with a timestamp.
    ///
    /// This translates to "returning" and "appearing" peers.
    pub returning_peer_with_timestamp: Option<String>,
}

impl From<&settings::AlertStrings> for AlertStringsConfig {
    /// Converts an `AlertStrings` into an `AlertStringsConfig` for serialization purposes.
    fn from(strings: &settings::AlertStrings) -> Self {
        Self {
            header: Some(strings.header.clone()),
            first_run_header: Some(strings.first_run_header.clone()),
            first_run_missing: Some(strings.first_run_missing.clone()),
            lost: Some(strings.lost.clone()),
            forgot: Some(strings.forgot.clone()),
            appeared: Some(strings.appeared.clone()),
            returned: Some(strings.returned.clone()),
            still_lost: Some(strings.still_lost.clone()),
            still_missing: Some(strings.still_missing.clone()),
            footer: Some(strings.footer.clone()),
            bullet_point: Some(strings.bullet_point.clone()),
            peer_with_timestamp: Some(strings.peer_with_timestamp.clone()),
            peer_no_timestamp: Some(strings.peer_no_timestamp.clone()),
            returning_peer_with_timestamp: Some(strings.returning_peer_with_timestamp.clone()),
        }
    }
}

/// Reminder message string configuration structure. This mirrors the runtime
/// settings struct used by the program for reminder message strings.
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ReminderStringsConfig {
    /// Message header for reminder notifications.
    pub header: Option<String>,

    /// Section header for peers that are still lost (timed out).
    pub still_lost: Option<String>,

    /// Section header for peers that have still not appeared since the last restart.
    pub still_missing: Option<String>,

    /// Message footer.
    pub footer: Option<String>,

    /// Bullet point string for listing peers.
    pub bullet_point: Option<String>,

    /// Message string for a peer with a timestamp, used in notifications
    /// when the timestamp of the last seen time is known.
    ///
    /// This translates to "lost" peers.
    pub peer_with_timestamp: Option<String>,

    /// Message string for a peer without a timestamp, used in notifications
    /// when the timestamp of the last seen time is zero.
    ///
    /// This translates to "missing" peers.
    pub peer_no_timestamp: Option<String>,
}

impl From<&settings::ReminderStrings> for ReminderStringsConfig {
    /// Converts a `ReminderStrings` into a `ReminderStringsConfig` for
    /// serialization purposes.
    fn from(strings: &settings::ReminderStrings) -> Self {
        Self {
            header: Some(strings.header.clone()),
            still_lost: Some(strings.still_lost.clone()),
            still_missing: Some(strings.still_missing.clone()),
            footer: Some(strings.footer.clone()),
            bullet_point: Some(strings.bullet_point.clone()),
            peer_with_timestamp: Some(strings.peer_with_timestamp.clone()),
            peer_no_timestamp: Some(strings.peer_no_timestamp.clone()),
        }
    }
}
