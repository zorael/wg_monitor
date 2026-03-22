//! Notification system for WG Monitor.
//!
//! This module provides the infrastructure for sending notifications and
//! reminders based on various events and conditions.
//!
//! The notification system is designed to be flexible and extensible,
//! allowing for different types of notifications and various backends for
//! sending notifications (e.g., Batsign, Slack, custom commands).

mod dispatch;
mod format;
mod notifier;
mod sender;
mod state;
mod stateful;
mod types;

pub use dispatch::{retry_pending_notifications, send_notification, send_reminder};
pub use format::{format_generic_message, format_generic_reminder};
pub use notifier::Notifier;
pub use sender::NotificationSender;
pub use state::NotifierState;
pub use stateful::StatefulNotifier;
pub use types::{Context, Delta, DispatchReport, NotificationResult, PendingNotification};
