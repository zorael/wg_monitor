//! Settings related to monitoring the WireGuard interface and connection status.

use std::time;

use crate::defaults;
use crate::file_config;

/// Settings for monitoring the WireGuard interface and connection status.
#[derive(Debug)]
pub struct MonitorSettings {
    /// WireGuard interface name to monitor.
    pub interface: String,

    /// Interval between checks of the WireGuard interface and connection status.
    pub check_interval: time::Duration,

    /// Timeout after which a peer is considered lost.
    pub timeout: time::Duration,

    /// Interval between reminders for lost peers.
    pub reminder_interval: time::Duration,

    /// Base retry interval for pending notifications.
    pub retry_interval: time::Duration,
}

impl Default for MonitorSettings {
    /// Default values for the monitor settings.
    fn default() -> Self {
        Self {
            interface: defaults::INTERFACE.to_string(),
            check_interval: defaults::CHECK_INTERVAL,
            timeout: defaults::TIMEOUT,
            reminder_interval: defaults::REMINDER_INTERVAL,
            retry_interval: defaults::RETRY_INTERVAL,
        }
    }
}

impl MonitorSettings {
    /// Apply settings from the file configuration, overriding defaults where
    /// values are available.
    pub fn apply_file(&mut self, monitor_config: &file_config::MonitorConfig) {
        if let Some(interface) = monitor_config.interface.clone() {
            self.interface = interface;
        }

        if let Some(check_interval) = monitor_config.check_interval {
            self.check_interval = check_interval;
        }

        if let Some(timeout) = monitor_config.timeout {
            self.timeout = timeout;
        }

        if let Some(reminder_interval) = monitor_config.reminder_interval {
            self.reminder_interval = reminder_interval;
        }

        if let Some(retry_interval) = monitor_config.retry_interval {
            self.retry_interval = retry_interval;
        }
    }

    /// Sanity check the monitor settings, appending any errors as strings to
    /// the passed vec.
    pub fn sanity_check(&self, vec: &mut Vec<String>) {
        if self.interface.is_empty() {
            vec.push("Monitor interface is not configured.".to_string());
        }

        if self.check_interval == time::Duration::ZERO {
            vec.push("Monitor check interval cannot be zero.".to_string());
        }

        if self.timeout == time::Duration::ZERO {
            vec.push("Monitor timeout cannot be zero.".to_string());
        }

        if self.reminder_interval == time::Duration::ZERO {
            vec.push("Monitor reminder interval cannot be zero.".to_string());
        }

        if self.retry_interval == time::Duration::ZERO {
            vec.push("Monitor retry interval cannot be zero.".to_string());
        }
    }
}
