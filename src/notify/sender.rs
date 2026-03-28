//! Notification sender trait.

/// Trait for sending notifications through a notifier backend.
pub trait NotificationSender {
    /// Returns the name of the notification sender, used for logging and reporting.
    ///
    /// The name is derived from the backend's name, and may include
    /// additional information such as unique identifiers to distinguish between
    /// multiple instances of the same backend type.
    fn name(&self) -> &str;

    /// Sends a notification based on the provided context and key delta.
    ///
    /// The context contains information about the current state of peers and
    /// timing, while the key delta represents the changes in peer status that
    /// triggered the notification.
    ///
    /// # Parameters
    /// - `ctx`: The current notification context, which contains information
    ///   about the peers and timing.
    /// - `delta`: The key delta representing the changes in peer status that
    ///   triggered the notification.
    /// # Returns
    /// A `NotificationResult` indicating the outcome of the notification attempt, which can be:
    /// - `DryRun(String)`: The notification was not sent because dry run mode is
    ///   enabled, but includes the message that would have been sent.
    /// - `Success(String, Option<String>)`: The notification was successfully
    ///   sent, including the message that was sent and any output from the backend.
    /// - `Failure(String, String)`: There was a failure in sending the notification,
    ///   including an error message describing the failure and the message that was attempted to be sent.
    /// - `NoMessage`: The notification was not sent because the rendered message ended up empty.
    /// - `Skipped`: The notification was skipped due to timing reasons.
    fn push_notification(
        &mut self,
        ctx: &super::Context,
        delta: &super::KeyDelta,
    ) -> super::NotificationResult;

    /// Sends a reminder notification based on the provided context.
    ///
    /// # Parameters
    /// - `ctx`: The current notification context, which contains information
    ///   about the peers and timing.
    ///
    /// # Returns
    /// A `NotificationResult` indicating the outcome of the reminder attempt, which can be:
    /// - `DryRun(String)`: The reminder was not sent because dry run mode is
    ///   enabled, but includes the message that would have been sent.
    /// - `Success(String, Option<String>)`: The reminder was successfully sent,
    ///   including the message that was sent and any output from the backend.
    /// - `Failure(String, String)`: There was a failure in sending the reminder,
    ///   including an error message describing the failure and the message that was attempted to be sent.
    /// - `NoMessage`: The reminder was not sent because the rendered message ended up empty
    /// - `Skipped`: The reminder was skipped due to timing reasons.
    fn push_reminder(&mut self, ctx: &super::Context) -> super::NotificationResult;

    /// Performs a sanity check on the notifier's backend.
    ///
    /// What this does is implementation-defined.
    ///
    /// # Returns
    /// - `Ok(())` if the sanity check passed without any issues.
    /// - `Err(Vec<String>)` if there were issues found during the sanity check,
    ///   containing a vector of descriptive error messages for each issue found.
    fn sanity_check(&self) -> Result<(), Vec<String>>;
}
