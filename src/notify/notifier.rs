//! FIXME

use crate::backend;

/// Defines the `NotificationSender` trait.
pub trait NotificationSender {
    /// Returns the name of the notifier, which is typically the name of the backend
    /// it uses (e.g., "slack" or "batsign") plus potentially any other identifier.
    fn name(&self) -> String;

    /// Sends a notification.
    fn push_notification(
        &mut self,
        ctx: &super::Context,
        delta: &super::Delta,
    ) -> super::NotificationResult;

    /// Sends a reminder notification.
    fn push_reminder(&mut self, ctx: &super::Context) -> super::NotificationResult;
}

/// A `Notifier` that uses a specific backend to send notifications about Wireguard peer status changes.
pub struct Notifier<B: backend::Backend> {
    /// The backend used to send notifications (e.g., Slack, Batsign).
    backend: B,

    /// Indicates whether the notifier is in dry run mode, which means that instead of
    /// actually sending notifications, it will print the messages to the console for testing purposes.
    dry_run: bool,
}

impl<B: backend::Backend> NotificationSender for Notifier<B> {
    /// Returns the name of the backend used by this notifier.
    fn name(&self) -> String {
        self.backend.name()
    }

    /// Sends a notification.
    fn push_notification(
        &mut self,
        ctx: &super::Context,
        delta: &super::Delta,
    ) -> super::NotificationResult {
        let message = self.backend.build_message(ctx, delta);

        if self.dry_run {
            return super::NotificationResult::DryRun(message);
        }

        match self.backend.send(&message) {
            Ok(_) => super::NotificationResult::Success,
            Err(e) => super::NotificationResult::Failure(e),
        }
    }

    /// Sends a reminder notification.
    fn push_reminder(&mut self, ctx: &super::Context) -> super::NotificationResult {
        let reminder = self.backend.build_reminder(ctx);

        if self.dry_run {
            return super::NotificationResult::DryRun(reminder);
        }

        match self.backend.send(&reminder) {
            Ok(_) => super::NotificationResult::Success,
            Err(e) => super::NotificationResult::Failure(e),
        }
    }
}

impl<B: backend::Backend> Notifier<B> {
    /// Creates a new `Notifier`.
    pub fn new(backend: B, dry_run: bool) -> Self {
        Self { backend, dry_run }
    }
}
