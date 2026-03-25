//! Types relating to pending notifications.

/// Enum representing a pending notification that is waiting to be retried.
#[derive(Debug, Clone)]
pub enum PendingNotification {
    Notification {
        context: super::Context,
        delta: super::KeyDelta,
    },
    Reminder {
        context: super::Context,
    },
}
