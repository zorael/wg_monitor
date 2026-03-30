//! Logic related to sending notifications about WireGuard peer status changes.

use crate::backend;

/// Notifier struct that holds the state and backend for sending notifications.
///
/// The `Notifier` is responsible for composing messages based on the
/// notification context and delta, and for sending the notifications through
/// the specified backend.
pub struct Notifier<B: backend::Backend> {
    /// State related to pending notifications, reminder timing, and failure tracking.
    pub state: super::NotifierState,

    /// The backend used to compose and send notifications.
    ///
    /// This can be any type that implements the `Backend` trait, currently one of
    /// `BatsignBackend`, `SlackBackend`, or `CommandBackend`.
    backend: B,

    /// Flag indicating whether the notifier is in dry run mode.
    ///
    /// If `true`, the notifier will compose messages but will not actually
    /// send them, and will return them as `DryRun` results.
    dry_run: bool,
}

impl<B: backend::Backend> Notifier<B> {
    /// Creates a new `Notifier` instance with the specified backend and dry-run mode.
    ///
    /// The dry-run mode is stored in the `Notifier` here, at instantiation.
    ///
    /// # Parameters
    /// - `backend`: The backend to use for composing and sending notifications.
    /// - `dry_run`: If `true`, the notifier will not actually send notifications,
    ///   but will return the composed messages as `DryRun` results.
    ///
    /// # Returns
    /// A new `Notifier` instance initialized with the provided backend and dry run mode.
    pub fn new(backend: B, dry_run: bool) -> Self {
        Self {
            backend,
            dry_run,
            state: super::NotifierState {
                failed_ctx: None,
                failed_delta: None,
                last_notification_sent: None,
                first_error_at: None,
                last_reminder_sent: None,
                last_failed_send: None,
                num_consecutive_reminders: 0,
                num_consecutive_failures: 0,
            },
        }
    }
}

impl<B: backend::Backend> super::NotificationSender for Notifier<B> {
    /// Returns the name of the notifier, which is derived from the backend's name.
    fn name(&self) -> &str {
        self.backend.name()
    }

    /// Sends a notification based on the provided context and delta.
    ///
    /// The method composes a message using the backend's `compose_message`
    /// method, and then sends it using the backend's `emit` method.
    ///
    /// The result of the notification attempt is returned as a
    /// `NotificationResult`, which can indicate success, failure, a dry run,
    /// or no message to send.
    ///
    /// # Parameters
    /// - `ctx`: The current notification context, which contains information
    ///   about the peers and timing.
    /// - `delta`: The delta representing the changes in peer status that
    ///   triggered the notification.
    ///
    /// # Returns
    /// A `NotificationResult` indicating the outcome of the notification
    /// attempt, which can be:
    /// - `DryRun(String)`: The notification was not sent because dry run mode is
    ///   enabled, but includes the message that would have been sent.
    /// - `Success(String, Option<String>)`: The notification was successfully sent,
    ///   including the message that was sent and any output from the backend.
    /// - `Failure(String, String)`: There was a failure in sending the notification,
    ///   including an error message describing the failure and the message
    ///   that was attempted to be sent.
    /// - `NoMessage`: The notification was not sent because the rendered
    ///   message ended up empty.
    fn push_notification(
        &mut self,
        ctx: &super::Context,
        delta: &super::KeyDelta,
    ) -> super::NotificationResult {
        let message = match self.backend.compose_message(ctx, delta) {
            Some(m) => m,
            None => return super::NotificationResult::NoMessage,
        };

        if self.dry_run {
            return super::NotificationResult::DryRun(message);
        }

        match self.backend.emit(ctx, Some(delta), &message) {
            Ok(output) => super::NotificationResult::Success(message, output),
            Err(e) => super::NotificationResult::Failure(e, message),
        }
    }

    /// Sends a reminder notification based on the provided context.
    ///
    /// The method composes a reminder message using the backend's
    /// `compose_reminder` method, and then sends it using the backend's `emit` method.
    ///
    /// The result of the reminder attempt is returned as a `NotificationResult`,
    /// which can indicate success, failure, a dry run, or no message to send.
    ///
    /// # Parameters
    /// - `ctx`: The current notification context, which contains information
    ///   about the peers and timing.
    ///
    /// # Returns
    /// A `NotificationResult` indicating the outcome of the reminder attempt,
    /// which can be:
    /// - `DryRun(String)`: The reminder was not sent because dry run mode is
    ///   enabled, but includes the message that would have been sent.
    /// - `Success(String, Option<String>)`: The reminder was successfully sent,
    ///   including the message that was sent and any output from the backend.
    /// - `Failure(String, String)`: There was a failure in sending the reminder,
    ///   including an error message describing the failure and the message
    ///   that was attempted to be sent.
    /// - `NoMessage`: The reminder was not sent because the rendered
    ///   message ended up empty.
    fn push_reminder(&mut self, ctx: &super::Context) -> super::NotificationResult {
        let reminder = match self.backend.compose_reminder(ctx) {
            Some(m) => m,
            None => return super::NotificationResult::NoMessage,
        };

        if self.dry_run {
            return super::NotificationResult::DryRun(reminder);
        }

        match self.backend.emit(ctx, None, &reminder) {
            Ok(output) => super::NotificationResult::Success(reminder, output),
            Err(e) => super::NotificationResult::Failure(e, reminder),
        }
    }

    /// Performs a sanity check on the backend.
    ///
    /// What this does is implementation-defined.
    ///
    /// # Returns
    /// - `Ok(())` if the sanity check passed without any issues.
    /// - `Err(Vec<String>)` if there were issues found during the sanity check,
    ///   containing a vector of descriptive error messages for each issue found.
    fn sanity_check(&self) -> Result<(), Vec<String>> {
        self.backend.sanity_check()
    }
}
