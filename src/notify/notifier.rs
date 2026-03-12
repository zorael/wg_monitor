//! Defines the `Notifier` struct, which implements the `NotificationSender`
//! trait and uses a backend to send notifications about Wireguard peer status changes.

use crate::backend;

/// A `Notifier` that uses a specific backend to send notifications about
/// Wireguard peer status changes.
pub struct Notifier<B: backend::Backend> {
    /// State related to pending notifications, reminder timing, and failure tracking.
    pub state: super::NotifierState,

    /// The backend used to send notifications (e.g., Slack, Batsign).
    backend: B,

    /// Whether the notifier is in dry run mode, where it builds the messages
    /// but does not actually send them, instead returning them as `DryRun` results.
    dry_run: bool,
}

impl<B: backend::Backend> Notifier<B> {
    /// Creates a new `Notifier`.
    pub fn new(backend: B, dry_run: bool) -> Self {
        Self {
            backend,
            dry_run,
            state: super::NotifierState {
                pending: None,
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
    /// Returns the name of the notifier, which is typically the name of the
    /// backend it uses (e.g., "slack" or "batsign") plus potentially any other
    /// unique identifiers.
    fn name(&self) -> &str {
        self.backend.name()
    }

    /// Sends a notification.
    fn push_notification(
        &mut self,
        ctx: &super::Context,
        delta: &super::Delta,
    ) -> super::NotificationResult {
        let message = self.backend.compose_message(ctx, delta);

        if self.dry_run {
            return super::NotificationResult::DryRun(message);
        }

        if message.is_empty() {
            return super::NotificationResult::NoMessage;
        }

        match self.backend.emit(ctx, Some(delta), &message) {
            Ok(Some(response)) => {
                println!("{response}");
                super::NotificationResult::Success(message)
            }
            Ok(None) => super::NotificationResult::Success(message),
            Err(e) => super::NotificationResult::Failure(e, message),
        }
    }

    /// Sends a reminder notification.
    fn push_reminder(&mut self, ctx: &super::Context) -> super::NotificationResult {
        let reminder = self.backend.compose_reminder(ctx);

        if self.dry_run {
            return super::NotificationResult::DryRun(reminder);
        }

        if reminder.is_empty() {
            return super::NotificationResult::Skipped;
        }

        match self.backend.emit(ctx, None, &reminder) {
            Ok(Some(response)) => {
                println!("{response}");
                super::NotificationResult::Success(reminder)
            }
            Ok(None) => super::NotificationResult::Success(reminder),
            Err(e) => super::NotificationResult::Failure(e, reminder),
        }
    }
}
