//! Result types used in the notification system.

/// Enum representing the result of a notification attempt.
#[derive(Debug)]
pub enum NotificationResult {
    /// Indicates that the notification was not actually sent because a dry run
    /// mode is in effect.
    ///
    /// Includes the composed message that would have been sent.
    DryRun(String),

    /// Indicates that the notification was successfully sent, including the
    /// composed message that was sent.
    Success(String),

    /// Indicates that there was a failure in sending the notification.
    ///
    /// Includes an error message describing the failure and the composed message
    /// that was attempted to be sent (in that order).
    Failure(String, String),

    /// Indicates that the notification was not sent because the composed message
    /// ended up empty.
    NoMessage,

    /// Indicates that the notification was skipped due to timing reasons.
    #[allow(dead_code)]
    Skipped,
}
