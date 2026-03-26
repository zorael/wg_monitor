//! Notification system.

mod dispatch;
mod format;
mod notifier;
mod sender;
mod state;
mod stateful;
mod types;

pub use dispatch::{retry_pending_notifications, send_notification, send_reminder};
pub use format::{prepare_message_body, prepare_reminder_body};
pub use notifier::Notifier;
pub use sender::NotificationSender;
pub use state::NotifierState;
pub use stateful::StatefulNotifier;
pub use types::{Context, DispatchReport, KeyDelta, NotificationResult, PendingNotification};
