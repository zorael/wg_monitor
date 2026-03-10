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

    /// The time when the last notification was successfully sent.
    pub last_notification_sent: Option<time::SystemTime>,

    /// The time when the first failure was recorded for the current pending notification.
    pub first_error_at: Option<time::SystemTime>,

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
    pub fn next_reminder_is_due(
        &self,
        now: &time::SystemTime,
        reminder_interval: &time::Duration,
    ) -> bool {
        let last_sent = match (
            self.last_reminder_sent,
            self.last_notification_sent,
            self.first_error_at,
        ) {
            (Some(last_reminder_sent), None, None) => {
                // A reminder has been recently sent so compare against that
                last_reminder_sent
            }
            (None, Some(last_notification_sent), None) => {
                // No reminder has been sent yet but a normal notification has
                // so compare against that
                last_notification_sent
            }
            (None, None, Some(first_error_at)) => {
                // A failure has been recorded but no reminder has been sent yet
                first_error_at
            }
            (None, None, None) => {
                // Nothing has been sent yet
                return false;
            }
            _ => {
                // Any other combination is an error state and should never happen
                return false;
            }
        };

        // Grow the reminder interval over time but cap it at 48h
        let growth_multiplier = match self.num_consecutive_reminders {
            0 => 1,  // 6h (base interval)
            1 => 2,  // 12h
            2 => 2,  // 12h
            3 => 4,  // 24h
            4 => 4,  // 24h
            5 => 8,  // 48h
            _ => 12, // 72h (cap)
        };

        let next_reminder_interval = reminder_interval.saturating_mul(growth_multiplier);

        match now.duration_since(last_sent) {
            Ok(duration) => duration >= next_reminder_interval,
            Err(_) => true, // Time went backwards?
        }
    }

    /// Checks if the next retry is due based on the time since the last failed
    /// send was recorded and the number of consecutive failures already recorded,
    /// using a growing interval.
    pub fn next_retry_is_due(
        &self,
        now: &time::SystemTime,
        retry_interval: &time::Duration,
    ) -> bool {
        let Some(last_failed_send) = self.last_failed_send else {
            // No send has failed yet, so there is nothing previous to delay against.
            // This implies that pending is None.
            return false;
        };

        let growth_multiplier = match self.num_consecutive_failures {
            0 => 1,  // 1m (base interval)
            1 => 2,  // 2m
            2 => 2,  // 2m
            3 => 5,  // 5m
            4 => 5,  // 5m
            _ => 10, // 10m (cap)
        };

        let next_retry_interval = retry_interval.saturating_mul(growth_multiplier);

        match now.duration_since(last_failed_send) {
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

        if self.first_error_at.is_none() {
            // Only update first_error_at if there is no error recorded yet
            self.first_error_at = Some(*now);
        }
    }

    /// Handles the logic for when a reminder is successfully sent, including
    /// updating the reminder timing and resetting failure tracking state.
    pub fn on_successful_reminder(&mut self, now: &time::SystemTime) {
        self.pending = None;
        self.last_notification_sent = None;
        self.last_reminder_sent = Some(*now);
        self.num_consecutive_reminders += 1;
        self.last_failed_send = None;
        self.num_consecutive_failures = 0;
        self.first_error_at = None;
    }

    /// Handles the logic for when a notification is successfully sent,
    /// resetting all state related to pending notifications, reminder timing,
    /// and failure tracking.
    pub fn on_successful_notification(&mut self, now: &time::SystemTime) {
        self.reset();
        self.last_notification_sent = Some(*now);
    }

    /// Resets all state related to pending notifications, reminder timing,
    /// and failure tracking.
    pub fn reset(&mut self) {
        self.pending = None;
        self.last_notification_sent = None;
        self.first_error_at = None;
        self.last_reminder_sent = None;
        self.last_failed_send = None;
        self.num_consecutive_reminders = 0;
        self.num_consecutive_failures = 0;
    }
}
