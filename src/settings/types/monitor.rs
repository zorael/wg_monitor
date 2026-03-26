//! Settings structures for monitoring the WireGuard interface and the
//! connection status of its peers.
//!
//! This module defines the `MonitorSettings` struct, which holds the runtime
//! settings for monitoring the WireGuard interface and connection status.

use std::time;

use crate::defaults;
use crate::file_config;

/// Settings structure for monitoring the WireGuard interface and connection status.
#[derive(Debug)]
pub struct MonitorSettings {
    /// The name of the WireGuard interface to monitor.
    pub interface: String,

    /// Interval for monitoring checks.
    pub check_interval: time::Duration,

    /// Timeout for monitoring checks, after which a peer is considered lost.
    pub timeout: time::Duration,

    /// Interval for reminder notifications for pending notifications.
    /// This will be grown as consecutive reminders are sent for the same
    /// pending notification.
    ///
    /// This allows for initially more frequent reminders,
    /// and less frequent reminders as time goes on without the peer status
    /// being resolved.
    pub reminder_interval: time::Duration,

    /// Interval for retrying failed notifications, which can help to ensure that
    /// notifications are eventually delivered even if there are temporary issues
    /// with the notification backends.
    pub retry_interval: time::Duration,
}

impl Default for MonitorSettings {
    /// Returns a `MonitorSettings` instance with default values for all fields,
    /// which are defined in the `defaults` module.
    ///
    /// This provides a baseline configuration that can be overridden by values
    /// from the file configuration.
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
    /// Applies settings from a `file_config::MonitorConfig` to the current
    /// `MonitorSettings` instance.
    ///
    /// # Parameters
    /// - `monitor_config`: The `file_config::MonitorConfig` containing the
    ///   settings to apply to the current `MonitorSettings` instance.
    pub fn apply_file(&mut self, monitor_config: &file_config::MonitorConfig) {
        if let Some(interface) = &monitor_config.interface {
            self.interface = interface.clone();
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

    /// Performs a sanity check on the monitor settings, validating that the
    /// interface name is not empty, and that the check interval, timeout,
    /// reminder interval, and retry interval are not zero.
    ///
    /// If any issues are found, descriptive error messages are added to the
    /// provided vector of strings.
    ///
    /// # Parameters
    /// - `vec`: A mutable reference to a vector of strings where error messages
    ///   will be added if any issues are found with the monitor settings.
    ///   If the settings are valid, this vector will remain unchanged.
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
