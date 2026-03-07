//! Module containing result types used in the notification system.

/// Represents the result of attempting to send a notification, which can be a
/// dry run, a success, or a failure with an error message.
#[derive(Debug)]
pub enum NotificationResult {
    /// Indicates that the notification was processed as a dry run, meaning
    /// no notification was sent.
    DryRun(String),

    /// Indicates that the notification was successful.
    Success(String),

    /// Indicates that the notification failed.
    Failure(String, String),

    /// Indicates that a stored notification was skipped.
    Skipped,
}
