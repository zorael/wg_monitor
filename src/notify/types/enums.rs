//! Enums used in notifications.

/// Enum representing a notification that can be stored for retrying later.
#[derive(Debug, Clone)]
pub enum StoredNotification {
    Notification(super::Context, super::Delta),
    Reminder(super::Context),
}
