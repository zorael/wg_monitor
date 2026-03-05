//! Module defining the `NotificationSender` trait and the `Notifier` struct,
//! which implements the trait using a specific backend to send notifications
//! about Wireguard peer status changes.

use std::time;

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

    fn store_notification(&mut self, ctx: &super::Context, delta: Option<&super::Delta>);

    fn get_stored_notification(&mut self) -> (Option<super::Context>, Option<super::Delta>);

    fn clear_stored_notification(&mut self);

    fn set_last_reminder_sent(&mut self, when: Option<time::SystemTime>);

    fn get_last_reminder_sent(&self) -> Option<time::SystemTime>;

    fn clear_last_reminder_sent(&mut self);

    fn get_num_consecutive_reminders(&self) -> u32;

    fn increment_num_consecutive_reminders(&mut self);

    fn reset_num_consecutive_reminders(&mut self);
}

/// A `Notifier` that uses a specific backend to send notifications about
/// Wireguard peer status changes.
pub struct Notifier<B: backend::Backend> {
    /// The backend used to send notifications (e.g., Slack, Batsign).
    backend: B,

    /// Indicates whether the notifier is in dry run mode, in which no
    /// notifications actually will be sent.
    dry_run: bool,

    stored_context: Option<super::Context>,

    stored_delta: Option<super::Delta>,

    last_reminder_sent: Option<time::SystemTime>,

    num_consecutive_reminders: u32,
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

    fn store_notification(&mut self, ctx: &super::Context, delta: Option<&super::Delta>) {
        self.stored_context = Some(ctx.clone());
        self.stored_delta = delta.cloned();
    }

    fn get_stored_notification(&mut self) -> (Option<super::Context>, Option<super::Delta>) {
        (self.stored_context.take(), self.stored_delta.take())
    }

    fn clear_stored_notification(&mut self) {
        self.stored_context = None;
        self.stored_delta = None;
    }

    fn set_last_reminder_sent(&mut self, when: Option<time::SystemTime>) {
        self.last_reminder_sent = when;
    }

    fn get_last_reminder_sent(&self) -> Option<time::SystemTime> {
        self.last_reminder_sent
    }

    fn clear_last_reminder_sent(&mut self) {
        self.last_reminder_sent = None;
    }

    fn get_num_consecutive_reminders(&self) -> u32 {
        self.num_consecutive_reminders
    }

    fn increment_num_consecutive_reminders(&mut self) {
        self.num_consecutive_reminders += 1;
    }

    fn reset_num_consecutive_reminders(&mut self) {
        self.num_consecutive_reminders = 0;
    }
}

impl<B: backend::Backend> Notifier<B> {
    /// Creates a new `Notifier`.
    pub fn new(backend: B, dry_run: bool) -> Self {
        Self {
            backend,
            dry_run,
            stored_context: None,
            stored_delta: None,
            last_reminder_sent: None,
            num_consecutive_reminders: 0,
        }
    }
}
