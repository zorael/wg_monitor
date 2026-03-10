//! Defines the `NotifierState` struct, which is used by notifiers to manage pending
//! notifications, reminder timing, and failure tracking.

use std::time;

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
    /// Saves a notification for later retrying, which can be either a regular
    /// notification with a context and delta, or a reminder with just a context.
    pub fn save_pending(&mut self, ctx: &super::Context, delta: Option<&super::Delta>) {
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
            2 => 4,  // 24h (1 day)
            3 => 8,  // 48h (2 days)
            4 => 12, // 72h (3 days)
            5 => 16, // 96h (4 days)
            _ => 28, // 168h (1 week, cap)
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
        self.save_pending(ctx, delta);
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
