//! State management for notifiers.

use std::time;

/// State struct for notifiers.
#[derive(Debug)]
pub struct NotifierState {
    /// The `Context` of one or more failed notification attempts.
    ///
    /// A `Context` that fails to be pushed is stored here. Further failed attempts
    /// will merge their `Context` with the existing one. This means the final
    /// notification may include conflicting information from multiple attempts,
    /// but that's the design for now.
    pub failed_ctx: Option<super::Context>,

    /// The `KeyDelta` of one or more failed notification attempts.
    ///
    /// A `KeyDelta` that fails to be pushed is stored here. Further failed attempts
    /// will merge their `KeyDelta` with the existing one. This means the final
    /// notification may include conflicting information from multiple attempts,
    /// but that's the design for now.
    pub failed_delta: Option<super::KeyDelta>,

    /// The time when the last notification was successfully sent, key in
    /// determining when the next reminder is due.
    pub last_notification_sent: Option<time::SystemTime>,

    /// The time when the first error was recorded for the current pending notification,
    /// used to determine how long the notification has been pending so
    /// the reminder interval can be grown over time.
    pub first_error_at: Option<time::SystemTime>,

    /// The time when the last reminder was sent, used to determine when the next
    /// reminder is due based on the reminder interval and the number of consecutive
    /// reminders already sent.
    pub last_reminder_sent: Option<time::SystemTime>,

    /// The time when the last failed send was recorded, used to determine when the next
    /// retry is due based on the retry interval and the number of consecutive
    /// failures already recorded.
    pub last_failed_send: Option<time::SystemTime>,

    pub num_consecutive_notifications: u32,

    /// The number of consecutive reminders sent for the current pending notification,
    /// used to grow the reminder interval over time.
    pub num_consecutive_reminders: u32,

    /// The number of consecutive failures recorded for the current pending notification,
    /// used to grow the retry interval over time.
    pub num_consecutive_failures: u32,
}

impl NotifierState {
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
    /// # Notes
    /// It currently uses a multiplier-based approach, and as such the growth
    /// will depend on the initial base reminder interval.
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
            0 => 1,  // 6h (...assuming a base interval of 6h)
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
    /// # Notes
    /// It currently uses a multiplier-based approach, and as such the growth
    /// will depend on the initial base retry interval.
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
            0..=11 => 1,  // 10s (...assuming a base interval of 10 seconds)
            12..=23 => 3, // 30s
            _ => 6,       // 1m (cap)
        };

        let next_retry_interval = retry_interval.saturating_mul(growth_multiplier);

        match now.duration_since(last_failed_send) {
            Ok(duration) => duration >= next_retry_interval,
            Err(_) => true, // Time went backwards?
        }
    }

    /// Handles the logic for when a notification or reminder fails to send.
    ///
    /// This includes saving the pending notification, updating the last failed send
    /// time, incrementing the number of consecutive failures, setting the
    /// first error time if it is not already set.
    ///
    /// # Notes
    /// This method should be called whenever a send attempt fails, regardless of
    /// whether it was a new notification or a reminder, to ensure that the state
    /// is updated correctly for retry and reminder timing.
    ///
    /// # Parameters
    /// - `ctx`: The notification context to save for the pending notification.
    /// - `delta`: An optional delta representing the changes in peer status that
    ///   triggered the notification. If `None`, this indicates that the pending
    ///   notification is a reminder rather than a new notification.
    pub fn on_failure(&mut self, ctx: &super::Context, delta: Option<&super::KeyDelta>) {
        self.last_failed_send = Some(ctx.now);
        self.num_consecutive_failures += 1;

        match &mut self.failed_ctx {
            Some(first_failed_ctx) => {
                first_failed_ctx.merge(ctx);
                first_failed_ctx.peers = ctx.peers.clone();
                first_failed_ctx.now = ctx.now;
            }
            None => {
                self.failed_ctx = Some(ctx.clone());
            }
        }

        match &mut self.failed_delta {
            Some(first_failed_delta) if delta.is_some() => {
                let delta = delta.expect("the arm guard verified that it is Some(KeyDelta)");
                first_failed_delta.merge(delta);
            }
            Some(_) => {}
            None => {
                self.failed_delta = delta.cloned();
            }
        }

        if self.first_error_at.is_none() {
            // Only update first_error_at if there is no error recorded yet
            self.first_error_at = Some(ctx.now);
        }
    }

    /// Handles the logic for when a reminder is successfully sent.
    ///
    /// This includes updating the
    /// last reminder sent time, incrementing the number of consecutive reminders,
    /// and resetting the failure tracking since a successful reminder indicates
    /// that the issue is still being actively worked on and should not be
    /// considered as failed for the purposes of retry timing.
    ///
    /// # Notes
    /// This method should be called whenever a reminder is successfully sent,
    /// regardless of whether it is the first reminder or a subsequent reminder,
    /// to ensure that the state is updated correctly for future reminder and
    /// retry timing.
    ///
    /// # Parameters
    /// - `now`: The current time.
    pub fn on_successful_reminder(&mut self, now: &time::SystemTime) {
        self.failed_ctx = None;
        self.failed_delta = None;
        self.last_notification_sent = None;
        self.last_reminder_sent = Some(*now);
        self.num_consecutive_reminders += 1;
        self.last_failed_send = None;
        self.num_consecutive_notifications = 0;
        self.num_consecutive_failures = 0;
        self.first_error_at = None;
    }

    /// Handles the logic for when a notification is successfully sent.
    ///
    /// This includes resetting
    /// all state related to pending notifications, reminder timing, and failure
    /// tracking, since a successful notification indicates that the issue has
    /// been resolved and there is no need to track any pending state or send
    /// reminders or retries.
    ///
    /// # Notes
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
        self.num_consecutive_notifications = 1;
    }

    /// Resets all state related to pending notifications, reminder timing, and
    /// failure tracking.
    pub fn reset(&mut self) {
        self.failed_ctx = None;
        self.failed_delta = None;
        self.last_notification_sent = None;
        self.first_error_at = None;
        self.last_reminder_sent = None;
        self.last_failed_send = None;
        self.num_consecutive_notifications = 0;
        self.num_consecutive_reminders = 0;
        self.num_consecutive_failures = 0;
    }
}
