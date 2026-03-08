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
                last_failed_send: None,
                num_consecutive_reminders: 0,
                num_consecutive_failures: 0,
            },
        }
    }
}

/// Trait for types that carry a `NotifierState`, allowing access to the state
/// for managing stored notifications, reminder timing, and failure tracking.
pub trait StateCarrier {
    /// Returns a reference to the `NotifierState`.
    fn state(&self) -> &NotifierState;

    /// Returns a mutable reference to the `NotifierState`.
    fn state_mut(&mut self) -> &mut NotifierState;
}

impl<B: backend::Backend> StateCarrier for Notifier<B> {
    /// Returns a reference to the `NotifierState`.
    fn state(&self) -> &NotifierState {
        &self.state
    }

    /// Returns a mutable reference to the `NotifierState`.
    fn state_mut(&mut self) -> &mut NotifierState {
        &mut self.state
    }
}

/// A `StatefulNotifier` is a `NotificationSender` that also carries a `NotifierState`,
/// allowing it to manage stored notifications, reminder timing, and failure tracking.
pub trait StatefulNotifier: NotificationSender + StateCarrier {}

/// Blanket implementation of `StatefulNotifier` for any type that implements both
/// `NotificationSender` and `StateCarrier`.
impl<T: NotificationSender + StateCarrier> StatefulNotifier for T {}

/// State carried by notifiers to manage stored notifications, reminder timing,
/// and failure tracking.
#[derive(Debug)]
pub struct NotifierState {
    /// An optional stored notification that failed to send, so it can be retried later.
    stored_notification: Option<super::StoredNotification>,

    /// The time when the last reminder was sent, used to determine when the
    /// next reminder is due.
    last_reminder_sent: Option<time::SystemTime>,

    /// The time when the last failed send was recorded, used to determine when
    /// the next retry is due.
    last_failed_send: Option<time::SystemTime>,

    /// The number of consecutive reminders sent for the current notification.
    num_consecutive_reminders: u32,

    /// The number of consecutive failures recorded for the current notification.
    num_consecutive_failures: u32,
}

impl NotifierState {
    /// Stores a notification for later retrying, which can be either a regular
    /// notification with a context and delta, or a reminder with just a context.
    pub fn store_notification(&mut self, ctx: &super::Context, delta: Option<&super::Delta>) {
        self.stored_notification = match delta {
            Some(d) => Some(super::StoredNotification::Notification(
                ctx.clone(),
                d.clone(),
            )),
            None => Some(super::StoredNotification::Reminder(ctx.clone())),
        }
    }

    /// Peeks at the stored notification without taking it, allowing the caller to
    /// see if there is a stored notification and what it is without modifying the state.
    #[allow(dead_code)]
    pub fn peek_stored_notification(&self) -> Option<&super::StoredNotification> {
        self.stored_notification.as_ref()
    }

    /// Takes the stored notification, removing it from the state and returning it.
    pub fn take_stored_notification(&mut self) -> Option<super::StoredNotification> {
        self.stored_notification.take()
    }

    /// Clears the stored notification without returning it, effectively discarding it.
    pub fn clear_stored_notification(&mut self) {
        self.stored_notification = None;
    }

    /// Sets the time when the last reminder was sent.
    pub fn set_last_reminder_sent(&mut self, when: Option<time::SystemTime>) {
        self.last_reminder_sent = when;
    }

    /// Peeks at the time when the last reminder was sent without modifying it.
    pub fn peek_last_reminder_sent(&self) -> Option<time::SystemTime> {
        self.last_reminder_sent
    }

    /// Clears the time when the last reminder was sent, effectively resetting
    /// the reminder timing.
    pub fn clear_last_reminder_sent(&mut self) {
        self.last_reminder_sent = None;
    }

    /// Sets the time when the last failed send was recorded.
    pub fn set_last_failed_send(&mut self, when: Option<time::SystemTime>) {
        self.last_failed_send = when;
    }

    /// Peeks at the time when the last failed send was recorded without modifying it.
    pub fn peek_last_failed_send(&self) -> Option<time::SystemTime> {
        self.last_failed_send
    }

    /// Clears the time when the last failed send was recorded, effectively resetting
    /// the retry timing.
    pub fn clear_last_failed_send(&mut self) {
        self.last_failed_send = None;
    }

    /// Gets the number of consecutive reminders sent by the notifier.
    pub fn get_num_consecutive_reminders(&self) -> u32 {
        self.num_consecutive_reminders
    }

    /// Increments the number of consecutive reminders sent by the notifier.
    pub fn increment_num_consecutive_reminders(&mut self) {
        self.num_consecutive_reminders += 1;
    }

    /// Resets the number of consecutive reminders sent by the notifier.
    pub fn reset_num_consecutive_reminders(&mut self) {
        self.num_consecutive_reminders = 0;
    }

    /// Checks if the next reminder is due based on the time since the last reminder
    /// was sent and the number of consecutive reminders already sent, using an
    /// exponentially growing interval.
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

        let next_reminder_interval = growth_multiplier * defaults::BASE_REMIND_INTERVAL;

        match now.duration_since(last_sent) {
            Ok(duration) => duration > next_reminder_interval,
            Err(_) => true, // Time went backwards?
        }
    }

    /// Gets the number of consecutive failures recorded by the notifier.
    pub fn get_num_consecutive_failures(&self) -> u32 {
        self.num_consecutive_failures
    }

    /// Increments the number of consecutive failures recorded by the notifier.
    pub fn increment_num_consecutive_failures(&mut self) {
        self.num_consecutive_failures += 1;
    }

    /// Resets the number of consecutive failures recorded by the notifier.
    pub fn reset_num_consecutive_failures(&mut self) {
        self.num_consecutive_failures = 0;
    }

    /// Checks if the next retry is due based on the time since the last failed
    /// send was recorded and the number of consecutive failures already recorded,
    /// using a growing interval.
    pub fn next_retry_is_due(&self, now: &time::SystemTime) -> bool {
        let Some(last_failed) = self.peek_last_failed_send() else {
            // No failed send has been recorded yet, meaning we're not in a retry context
            return false;
        };

        let growth_multiplier = match self.get_num_consecutive_failures() {
            0 => 1,  // 1m (base interval)
            1 => 5,  // 5m
            _ => 10, // 10m
        };

        let next_retry_interval = growth_multiplier * defaults::BASE_RETRY_INTERVAL;

        match now.duration_since(last_failed) {
            Ok(duration) => duration > next_retry_interval,
            Err(_) => true, // Time went backwards?
        }
    }

    /// Resets all state related to stored notifications, reminder timing,
    /// and failure tracking.
    pub fn reset(&mut self) {
        self.clear_stored_notification();
        self.clear_last_reminder_sent();
        self.clear_last_failed_send();
        self.reset_num_consecutive_reminders();
        self.reset_num_consecutive_failures();
    }
}
