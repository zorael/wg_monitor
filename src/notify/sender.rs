//! Notification sender trait.

/// Trait for sending notifications through a notifier backend.
pub trait NotificationSender {
    /// Returns the name of the notification sender, used for logging and reporting.
    ///
    /// The name is typically derived from the backend's name, and may include
    /// additional information such as unique identifiers to distinguish between
    /// multiple instances of the same backend type.
    ///
    /// # Returns
    /// A string slice representing the name of this notification sender.
    fn name(&self) -> &str;

    /// Sends a notification based on the provided context and key delta.
    ///
    /// The context contains information about the current state of peers and
    /// timing, while the key delta represents the changes in peer status that
    /// triggered the notification.
    ///
    /// The method returns a `NotificationResult` indicating the outcome of the
    /// notification attempt, which can include success, failure, or dry run
    /// results, as well as any messages or errors associated with the attempt.
    ///
    /// # Parameters
    /// - `ctx`: The current notification context, which contains information
    ///   about the peers and timing.
    /// - `delta`: The key delta representing the changes in peer status that
    ///   triggered the notification.
    /// # Returns
    /// A `NotificationResult` indicating the outcome of the notification attempt, which can be:
    /// - `DryRun(message)`: The notification was not sent because dry run mode is
    ///   enabled, but includes the message that would have been sent.
    /// - `Success(message)`: The notification was successfully sent, including the message that was sent.
    /// - `Failure(error, message)`: There was a failure in sending the notification,
    ///   including an error message describing the failure and the message that was attempted to be sent.
    /// - `NoMessage`: The notification was not sent because the rendered message ended up empty.
    /// - `Skipped`: The notification was skipped, typically due to timing reasons.
    fn push_notification(
        &mut self,
        ctx: &super::Context,
        delta: &super::KeyDelta,
    ) -> super::NotificationResult;

    /// Sends a reminder notification based on the provided context.
    ///
    /// The method returns a `NotificationResult` indicating the outcome of the
    /// reminder attempt, which can include success, failure, or dry run results,
    /// as well as any messages or errors associated with the attempt.
    ///
    /// # Parameters
    /// - `ctx`: The current notification context, which contains information
    ///   about the peers and timing.
    ///
    /// # Returns
    /// A `NotificationResult` indicating the outcome of the reminder attempt, which can be:
    /// - `DryRun(message)`: The reminder was not sent because dry run mode is
    ///   enabled, but includes the message that would have been sent.
    /// - `Success(message)`: The reminder was successfully sent, including the message that was
    ///   sent.
    /// - `Failure(error, message)`: There was a failure in sending the reminder,
    ///   including an error message describing the failure and the message that was attempted to be sent.
    /// - `NoMessage`: The reminder was not sent because the rendered message ended up empty
    /// - `Skipped`: The reminder was skipped, typically due to timing reasons.
    fn push_reminder(&mut self, ctx: &super::Context) -> super::NotificationResult;
}
