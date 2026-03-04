//! Configuration file structure for the base monitoring functionality of the
//! program, which can be deserialized from a configuration file on disk.

use serde::{Deserialize, Serialize};
use std::time;

/// Monitor configuration structures for the program. This mirrors the runtime
/// settings struct used by the program for monitoring the Wireguard interface.
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MonitorConfig {
    /// Wireguard interface name to monitor.
    pub interface: Option<String>,

    /// Check interval for monitoring the Wireguard interface,
    /// specified as a human-readable duration (e.g., "30s", "1m").
    #[serde(with = "humantime_serde")]
    pub check_interval: Option<time::Duration>,

    /// Timeout for monitoring checks, specified as a human-readable duration.
    /// If the time since a given peer exceeds this timeout, the peer is considered missing.
    #[serde(with = "humantime_serde")]
    pub timeout: Option<time::Duration>,

    /// Base reminder interval for sending reminder notifications about missing
    /// peers, specified as a human-readable duration. This duration will be
    /// grown as consecutive reminders are sent.
    #[serde(with = "humantime_serde")]
    pub reminder_interval: Option<time::Duration>,
}
