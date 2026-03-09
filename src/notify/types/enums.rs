//! Enums used in notifications.

/// Enum representing a pending notification that can be stored for retrying later.
#[derive(Debug, Clone)]
pub enum PendingNotification {
    Notification(super::Context, super::Delta),
    Reminder(super::Context),
}
