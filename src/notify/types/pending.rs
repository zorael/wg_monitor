//! Types relating to pending notifications.

/// Enum representing a pending notification that is waiting to be retried.
#[derive(Debug, Clone)]
pub enum PendingNotification {
    /// A pending notification that is waiting to be retried, carrying the context
    /// and delta of the notification.
    Notification {
        context: super::Context,
        delta: super::KeyDelta,
    },

    /// A pending reminder notification that is waiting to be retried, carrying
    /// the context of the reminder.
    Reminder { context: super::Context },
}
