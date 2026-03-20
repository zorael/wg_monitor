//! Monitor configuration struct for the file-based configuration system.
//!
//! This struct mirrors the runtime settings struct used by the program for
//! monitoring the WireGuard interface, and can be deserialized from the
//! configuration file on disk.

use serde::{Deserialize, Serialize};
use std::time;

/// Monitor configuration structure.
///
/// This mirrors the runtime settings struct used by the program for monitoring
/// the WireGuard interface.
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MonitorConfig {
    /// The name of the WireGuard interface to monitor.
    pub interface: Option<String>,

    /// Interval for monitoring checks, specified as a human-readable duration.
    ///
    /// This is the interval at which the program checks the status of the
    /// peers on the WireGuard interface.
    #[serde(with = "humantime_serde")]
    pub check_interval: Option<time::Duration>,

    /// Timeout for monitoring checks, specified as a human-readable duration.
    #[serde(with = "humantime_serde")]
    pub timeout: Option<time::Duration>,

    /// Interval for reminder notifications, specified as a human-readable duration.
    ///
    /// This is the interval at which reminder notifications are sent.
    /// It will be grown as consecutive reminders are sent for the same pending
    /// notification. This allows for more frequent reminders at the beginning,
    /// and less frequent reminders as time goes on without the pending
    /// notification being resolved.
    #[serde(with = "humantime_serde")]
    pub reminder_interval: Option<time::Duration>,

    /// Interval for retrying failed notifications, specified as a human-readable duration.
    #[serde(with = "humantime_serde")]
    pub retry_interval: Option<time::Duration>,
}
