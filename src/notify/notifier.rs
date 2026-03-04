//! Module defining the `NotificationSender` trait and the `Notifier` struct,
//! which implements the trait using a specific backend to send notifications
//! about Wireguard peer status changes.

use crate::backend;

/// Defines the `NotificationSender` trait, implemented by types that can send
/// notifications about Wireguard peer status changes.
pub trait NotificationSender {
    /// Returns the name of the notifier, which is typically the name of the
    /// backend it uses (e.g., "slack" or "batsign") plus potentially any other
    /// unique identifiers.
    fn name(&self) -> String;

    /// Sends a notification.
    fn push_notification(
        &mut self,
        ctx: &super::Context,
        delta: &super::Delta,
    ) -> (super::NotificationResult, String);

    /// Sends a reminder notification.
    fn push_reminder(&mut self, ctx: &super::Context) -> (super::NotificationResult, String);
}

/// A `Notifier` that uses a specific backend to send notifications about
/// Wireguard peer status changes.
pub struct Notifier<B: backend::Backend> {
    /// The backend used to send notifications (e.g., Slack, Batsign).
    backend: B,

    /// Indicates whether the notifier is in dry run mode, in which no
    /// notifications actually will be sent.
    dry_run: bool,
}

impl<B: backend::Backend> NotificationSender for Notifier<B> {
    /// Returns the name of the notifier, which is typically the name of the
    /// backend it uses (e.g., "slack" or "batsign") plus potentially any other
    /// unique identifiers.
    fn name(&self) -> String {
        self.backend.name()
    }

    /// Sends a notification.
    fn push_notification(
        &mut self,
        ctx: &super::Context,
        delta: &super::Delta,
    ) -> (super::NotificationResult, String) {
        let message = self.backend.build_message(ctx, delta);

        if self.dry_run {
            return (super::NotificationResult::DryRun, message);
        }

        match self.backend.send(&message) {
            Ok(_) => (super::NotificationResult::Success, message),
            Err(e) => (super::NotificationResult::Failure(e), message),
        }
    }

    /// Sends a reminder notification.
    fn push_reminder(&mut self, ctx: &super::Context) -> (super::NotificationResult, String) {
        let reminder = self.backend.build_reminder(ctx);

        if self.dry_run {
            return (super::NotificationResult::DryRun, reminder);
        }

        match self.backend.send(&reminder) {
            Ok(_) => (super::NotificationResult::Success, reminder),
            Err(e) => (super::NotificationResult::Failure(e), reminder),
        }
    }
}

impl<B: backend::Backend> Notifier<B> {
    /// Creates a new `Notifier`.
    pub fn new(backend: B, dry_run: bool) -> Self {
        Self { backend, dry_run }
    }
}
