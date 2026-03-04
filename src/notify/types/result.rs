//! FIXME

/// Represents the result of attempting to send a notification, which can be a
/// dry run (message printed to console), a success, or a failure with an error message.
pub enum NotificationResult {
    /// Indicates that the notification was processed as a dry run, meaning
    /// it was printed to the console instead of being sent.
    DryRun(String),

    /// Indicates that the notification was successful.
    Success,

    /// Indicates that the notification failed.
    Failure(String),
}
