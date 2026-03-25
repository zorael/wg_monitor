//! State management for notifiers.

use std::time;

/// State struct for notifiers, tracking pending notifications, reminder timing,
/// and failure tracking to manage the notification flow effectively.
#[derive(Debug)]
pub struct NotifierState {
    /// An optional pending notification that failed to send, so it can be retried later.
    pub pending: Option<super::PendingNotification>,

    /// The time when the last notification was successfully sent, key in
    /// determining when the next reminder is due.
    pub last_notification_sent: Option<time::SystemTime>,

    /// The time when the first error was recorded for the current pending notification,
    /// used to determine how long the notification has been pending and to grow
    /// the reminder interval over time.
    pub first_error_at: Option<time::SystemTime>,

    /// The time when the last reminder was sent, used to determine when the next
    /// reminder is due based on the reminder interval and the number of consecutive
    /// reminders already sent.
    pub last_reminder_sent: Option<time::SystemTime>,

    /// The time when the last failed send was recorded, used to determine when the next
    /// retry is due based on the retry interval and the number of consecutive
    /// failures already recorded.
    pub last_failed_send: Option<time::SystemTime>,

    /// The number of consecutive reminders sent for the current pending notification,
    /// used to grow the reminder interval over time.
    pub num_consecutive_reminders: u32,

    /// The number of consecutive failures recorded for the current pending notification,
    /// used to grow the retry interval over time.
    pub num_consecutive_failures: u32,
}

impl NotifierState {
    /// Saves the pending notification in the state, which can be either a
    /// normal notification (with a delta) or a reminder (without a delta),
    /// based on the provided context and optional delta.
    ///
    /// The pending notification is stored in the `pending` field of the state,
    /// wrapped in the appropriate `PendingNotification` variant based on
    /// whether a delta is provided or not.
    ///
    /// # Parameters
    /// - `ctx`: The notification context to save for the pending notification.
    /// - `delta`: An optional delta representing the changes in peer status that
    ///   triggered the notification. If `None`, this indicates that the pending
    ///   notification is a reminder rather than a new alert.
    pub fn save_pending(&mut self, ctx: &super::Context, delta: Option<&super::KeyDelta>) {
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

    /// Checks if the next reminder is due based on the time since the last
    /// reminder or notification was sent (or the first error was recorded if
    /// no reminder or notification has been sent yet) and the number of
    /// consecutive reminders already sent, using a growing interval.
    ///
    /// The reminder interval grows over time to avoid sending reminders too
    /// frequently for notifications that have been pending for a long time,
    /// but it is capped at a maximum interval to ensure that reminders are
    /// still sent eventually even for notifications that have been pending
    /// for a very long time.
    ///
    /// # Parameters
    /// - `now`: The current time to compare against the last reminder sent,
    ///   last notification sent, or first error recorded to determine if the
    ///   next reminder is due.
    /// - `reminder_interval`: The base interval to use for calculating when
    ///   the next reminder is due, which will be multiplied by a growth factor
    ///   based on the number of consecutive reminders already sent.
    ///
    /// # Returns
    /// - `true` if the next reminder is due based on the time since the last
    ///   reminder sent, last notification sent, or first error recorded and
    ///   the calculated next reminder interval.
    /// - `false` if the next reminder is not yet due based on the time since
    ///   the last relevant event and the calculated next reminder interval.
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

    /// Checks if the next retry is due based on the time since the last
    /// failed send was recorded and the number of consecutive failures already
    /// recorded, using a growing interval.
    ///
    /// The retry interval grows over time to avoid sending retries too
    /// frequently for notifications that have been pending for a long time,
    /// but it is capped at a maximum interval to ensure that retries are
    /// still attempted eventually even for notifications that have been pending
    /// for a very long time.
    ///
    /// # Parameters
    /// - `now`: The current time to compare against the last failed send to
    ///   determine if the next retry is due.
    /// - `retry_interval`: The base interval to use for calculating when
    ///   the next retry is due, which will be multiplied by a growth factor
    ///   based on the number of consecutive failures already recorded.
    ///
    /// # Returns
    /// - `true` if the next retry is due based on the time since the last
    ///   failed send and the calculated next retry interval.
    /// - `false` if the next retry is not yet due based on the time since
    ///   the last failed send and the calculated next retry interval.
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

    /// Handles the logic for when a notification or reminder fails to send,
    /// including saving the pending notification, updating the last failed send
    /// time, incrementing the number of consecutive failures, and setting the
    /// first error time if it is not already set.
    ///
    /// This method should be called whenever a send attempt fails, regardless of
    /// whether it was a new notification or a reminder, to ensure that the state
    /// is updated correctly for retry and reminder timing.
    ///
    /// # Parameters
    /// - `ctx`: The notification context to save for the pending notification.
    /// - `delta`: An optional delta representing the changes in peer status that
    ///   triggered the notification. If `None`, this indicates that the pending
    ///   notification is a reminder rather than a new notification.
    /// - `now`: The current time to record as the last failed send time and to
    ///   set as the first error time if it is not already set.
    pub fn on_failure(
        &mut self,
        ctx: &super::Context,
        delta: Option<&super::KeyDelta>,
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

    /// Handles the logic for when a reminder is successfully sent, updating the
    /// last reminder sent time, incrementing the number of consecutive reminders,
    /// and resetting the failure tracking since a successful reminder indicates
    /// that the issue is still being actively worked on and should not be
    /// considered as failed for the purposes of retry timing.
    ///
    /// This method should be called whenever a reminder is successfully sent,
    /// regardless of whether it is the first reminder or a subsequent reminder,
    /// to ensure that the state is updated correctly for future reminder and
    /// retry timing.
    ///
    /// # Parameters
    /// - `now`: The current time.
    pub fn on_successful_reminder(&mut self, now: &time::SystemTime) {
        self.pending = None;
        self.last_notification_sent = None;
        self.last_reminder_sent = Some(*now);
        self.num_consecutive_reminders += 1;
        self.last_failed_send = None;
        self.num_consecutive_failures = 0;
        self.first_error_at = None;
    }

    /// Handles the logic for when a notification is successfully sent, resetting
    /// all state related to pending notifications, reminder timing, and failure
    /// tracking, since a successful notification indicates that the issue has
    /// been resolved and there is no need to track any pending state or send
    /// reminders or retries.
    ///
    /// This method should be called whenever a notification is successfully
    /// sent, regardless of whether it is the first notification or a subsequent
    /// notification, to ensure that the state is updated correctly and all
    /// pending, reminder, and failure tracking is reset for future notifications.
    ///
    /// # Parameters
    /// - `now`: The current time.
    pub fn on_successful_notification(&mut self, now: &time::SystemTime) {
        self.reset();
        self.last_notification_sent = Some(*now);
    }

    /// Resets all state related to pending notifications, reminder timing, and
    /// failure tracking.
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
