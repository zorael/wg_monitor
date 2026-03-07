//! The `notify` module contains all the logic related to sending notifications about
//! Wireguard peer status changes, including building messages based on the notification
//! context and delta, and dispatching notifications to all configured notifiers.

mod dispatch;
mod format;
mod traits;
mod types;

pub use dispatch::{retry_stored_notification, send_notification, send_single_notifier_reminder};
pub use format::{format_generic_message, format_generic_reminder};
pub use traits::{/*NotificationSender,*/ Notifier, StatefulNotifier};
pub use types::{Context, Delta, DispatchReport, NotificationResult, StoredNotification};
