//! Module defining the `NotificationSender` trait and the `Notifier` struct,
//! which implements the trait using a specific backend to send notifications
//! about Wireguard peer status changes.

use std::time;

use crate::backend;
use crate::defaults;

/// Defines the `NotificationSender` trait, implemented by types that can send
/// notifications about Wireguard peer status changes.
pub trait NotificationSender {
    /// Returns the name of the notifier, which is typically the name of the
    /// backend it uses (e.g., "slack" or "batsign") plus potentially any other
    /// unique identifiers.
    fn name(&mut self) -> &str;

    /// Sends a notification.
    fn push_notification(
        &mut self,
        ctx: &super::Context,
        delta: &super::Delta,
    ) -> super::NotificationResult;

    /// Sends a reminder notification.
    fn push_reminder(&mut self, ctx: &super::Context) -> super::NotificationResult;
}

/// A `Notifier` that uses a specific backend to send notifications about
/// Wireguard peer status changes.
pub struct Notifier<B: backend::Backend> {
    pub state: NotifierState,

    /// The backend used to send notifications (e.g., Slack, Batsign).
    backend: B,

    dry_run: bool,
}

impl<B: backend::Backend> NotificationSender for Notifier<B> {
    /// Returns the name of the notifier, which is typically the name of the
    /// backend it uses (e.g., "slack" or "batsign") plus potentially any other
    /// unique identifiers.
    fn name(&mut self) -> &str {
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
            Ok(_) => super::NotificationResult::Success(message),
            Err(e) => super::NotificationResult::Failure(e, message),
        }
    }

    /// Sends a reminder notification.
    fn push_reminder(&mut self, ctx: &super::Context) -> super::NotificationResult {
        let reminder = self.backend.build_reminder(ctx);

        if self.dry_run {
            return super::NotificationResult::DryRun(reminder);
        }

        match self.backend.send(&reminder) {
            Ok(_) => super::NotificationResult::Success(reminder),
            Err(e) => super::NotificationResult::Failure(e, reminder),
        }
    }
}

impl<B: backend::Backend> Notifier<B> {
    /// Creates a new `Notifier`.
    pub fn new(backend: B, dry_run: bool) -> Self {
        Self {
            backend,
            dry_run,
            state: NotifierState {
                stored_notification: None,
                last_reminder_sent: None,
                num_consecutive_reminders: 0,
            },
        }
    }
}

pub trait StateCarrier {
    fn state(&self) -> &NotifierState;
    fn state_mut(&mut self) -> &mut NotifierState;
}

impl<B: backend::Backend> StateCarrier for Notifier<B> {
    fn state(&self) -> &NotifierState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut NotifierState {
        &mut self.state
    }
}

pub trait StatefulNotifier: NotificationSender + StateCarrier {}
impl<T: NotificationSender + StateCarrier> StatefulNotifier for T {}

pub struct NotifierState {
    stored_notification: Option<super::StoredNotification>,

    last_reminder_sent: Option<time::SystemTime>,

    num_consecutive_reminders: u32,
}

impl NotifierState {
    pub fn store_notification(&mut self, ctx: &super::Context, delta: Option<&super::Delta>) {
        self.stored_notification = match delta {
            Some(d) => Some(super::StoredNotification::Notification(
                ctx.clone(),
                d.clone(),
            )),
            None => Some(super::StoredNotification::Reminder(ctx.clone())),
        }
    }

    #[allow(dead_code)]
    pub fn peek_stored_notification(&self) -> Option<&super::StoredNotification> {
        self.stored_notification.as_ref()
    }

    pub fn take_stored_notification(&mut self) -> Option<super::StoredNotification> {
        self.stored_notification.take()
    }

    pub fn clear_stored_notification(&mut self) {
        self.stored_notification = None;
    }

    pub fn set_last_reminder_sent(&mut self, when: Option<time::SystemTime>) {
        self.last_reminder_sent = when;
    }

    pub fn peek_last_reminder_sent(&self) -> Option<time::SystemTime> {
        self.last_reminder_sent
    }

    pub fn clear_last_reminder_sent(&mut self) {
        self.last_reminder_sent = None;
    }

    pub fn get_num_consecutive_reminders(&self) -> u32 {
        self.num_consecutive_reminders
    }

    pub fn increment_num_consecutive_reminders(&mut self) {
        self.num_consecutive_reminders += 1;
    }

    pub fn reset_num_consecutive_reminders(&mut self) {
        self.num_consecutive_reminders = 0;
    }

    pub fn next_reminder_is_due(&self, now: &time::SystemTime) -> bool {
        let Some(last_sent) = self.peek_last_reminder_sent() else {
            // No reminder has been sent yet, meaning we're not in a reminder context
            return false;
        };

        // Grow the reminder interval over time but cap it at 48h
        let growth_multiplier = match self.get_num_consecutive_reminders() {
            0 => 1, // 6h (base interval)
            1 => 2, // 12h
            2 => 2, // 12h
            3 => 4, // 24h
            4 => 4, // 24h
            _ => 8, // 48h
        };

        let next_report_interval = growth_multiplier * defaults::BASE_RETRY_INTERVAL;

        match now.duration_since(last_sent) {
            Ok(duration) => duration > next_report_interval,
            Err(_) => true, // Time went backwards?
        }
    }
}
