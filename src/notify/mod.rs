//! The `notify` module contains all the logic related to sending notifications about
//! Wireguard peer status changes, including building messages based on the notification
//! context and delta, and dispatching notifications to all configured notifiers.

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
