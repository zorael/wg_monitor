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
    fn name(&self) -> &str;

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
    /// State related to pending notifications, reminder timing, and failure tracking.
    pub state: NotifierState,

    /// The backend used to send notifications (e.g., Slack, Batsign).
    backend: B,

    /// Whether the notifier is in dry run mode, where it builds the messages
    /// but does not actually send them, instead returning them as `DryRun` results.
    dry_run: bool,
}

impl<B: backend::Backend> NotificationSender for Notifier<B> {
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
                pending: None,
                last_reminder_sent: None,
                last_failed_send: None,
                num_consecutive_reminders: 0,
                num_consecutive_failures: 0,
            },
        }
    }
}

/// Trait for types that carry a `NotifierState`, allowing access to the state
/// for managing pending notifications, reminder timing, and failure tracking.
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
/// allowing it to manage pending notifications, reminder timing, and failure tracking.
pub trait StatefulNotifier: NotificationSender + StateCarrier {}

/// Blanket implementation of `StatefulNotifier` for any type that implements both
/// `NotificationSender` and `StateCarrier`.
impl<T: NotificationSender + StateCarrier> StatefulNotifier for T {}

/// State carried by notifiers to manage pending notifications, reminder timing,
/// and failure tracking.
#[derive(Debug)]
pub struct NotifierState {
    /// An optional pending notification that failed to send, so it can be retried later.
    pub pending: Option<super::PendingNotification>,

    /// The time when the last reminder was sent, used to determine when the
    /// next reminder is due.
    pub last_reminder_sent: Option<time::SystemTime>,

    /// The time when the last failed send was recorded, used to determine when
    /// the next retry is due.
    pub last_failed_send: Option<time::SystemTime>,

    /// The number of consecutive reminders sent for the current notification.
    pub num_consecutive_reminders: u32,

    /// The number of consecutive failures recorded for the current notification.
    pub num_consecutive_failures: u32,
}

impl NotifierState {
    /// Stores a notification for later retrying, which can be either a regular
    /// notification with a context and delta, or a reminder with just a context.
    pub fn store_notification(&mut self, ctx: &super::Context, delta: Option<&super::Delta>) {
        self.pending = match delta {
            Some(d) => Some(super::PendingNotification::Notification {
                context: ctx.clone(),
                delta: d.clone(),
            }),
            None => Some(super::PendingNotification::Reminder {
                context: ctx.clone(),
            }),
        }
    }

    /// Checks if the next reminder is due based on the time since the last reminder
    /// was sent and the number of consecutive reminders already sent, using an
    /// exponentially growing interval.
    pub fn next_reminder_is_due(&self, now: &time::SystemTime) -> bool {
        let Some(last_sent) = self.last_reminder_sent else {
            // No reminder has been sent yet, meaning we're not in a reminder context
            return false;
        };

        // Grow the reminder interval over time but cap it at 48h
        let growth_multiplier = match self.num_consecutive_reminders {
            0 => 1, // 6h (base interval)
            1 => 2, // 12h
            2 => 2, // 12h
            3 => 4, // 24h
            4 => 4, // 24h
            _ => 8, // 48h
        };

        let next_reminder_interval =
            defaults::BASE_REMINDER_INTERVAL.saturating_mul(growth_multiplier);

        match now.duration_since(last_sent) {
            Ok(duration) => duration >= next_reminder_interval,
            Err(_) => true, // Time went backwards?
        }
    }

    /// Checks if the next retry is due based on the time since the last failed
    /// send was recorded and the number of consecutive failures already recorded,
    /// using a growing interval.
    pub fn next_retry_is_due(&self, now: &time::SystemTime) -> bool {
        let Some(last_failed) = self.last_failed_send else {
            // No failed send has been recorded yet, meaning we're not in a retry context
            return false;
        };

        let growth_multiplier = match self.num_consecutive_failures {
            0 => 1,  // 1m (base interval)
            1 => 5,  // 5m
            _ => 10, // 10m
        };

        let next_retry_interval = defaults::BASE_RETRY_INTERVAL.saturating_mul(growth_multiplier);

        match now.duration_since(last_failed) {
            Ok(duration) => duration >= next_retry_interval,
            Err(_) => true, // Time went backwards?
        }
    }

    /// Handles the logic for when a send failure occurs, including storing the
    /// notification for retrying and updating the failure tracking state.
    pub fn on_failure(
        &mut self,
        ctx: &super::Context,
        delta: Option<&super::Delta>,
        now: &time::SystemTime,
    ) {
        self.store_notification(ctx, delta);
        self.last_failed_send = Some(*now);
        self.num_consecutive_failures += 1;
    }

    /// Handles the logic for when a reminder is successfully sent, including
    /// updating the reminder timing and resetting failure tracking state.
    pub fn on_successful_reminder(&mut self, now: &time::SystemTime) {
        self.pending = None; // Not necessary if it was take()n in calling code
        self.last_reminder_sent = Some(*now);
        self.num_consecutive_reminders += 1;
        self.last_failed_send = None;
        self.num_consecutive_failures = 0;
    }

    /// Handles the logic for when a notification is successfully sent,
    /// resetting all state related to pending notifications, reminder timing,
    /// and failure tracking.
    pub fn on_successful_notification(&mut self) {
        self.reset()
    }

    /// Resets all state related to pending notifications, reminder timing,
    /// and failure tracking.
    pub fn reset(&mut self) {
        self.pending = None;
        self.last_reminder_sent = None;
        self.last_failed_send = None;
        self.num_consecutive_reminders = 0;
        self.num_consecutive_failures = 0;
    }
}
