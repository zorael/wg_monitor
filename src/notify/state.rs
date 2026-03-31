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

    /// The time when the last alert was successfully sent, key in
    /// determining when the next reminder is due.
    pub last_alert_sent: Option<time::SystemTime>,

    /// The time when the last reminder was sent.
    pub last_reminder_sent: Option<time::SystemTime>,

    /// The time when the last send failed.
    pub last_failed_send: Option<time::SystemTime>,

    /// The number of consecutive reminders sent.
    pub num_consecutive_reminders: u32,

    /// The number of consecutive failures recorded.
    pub num_consecutive_failures: u32,
}

impl NotifierState {
    /// Checks if the next reminder is due, using a growing interval.
    ///
    /// The reminder interval grows over time to avoid sending reminders too
    /// frequently for notifications that have been unresolved for a long time,
    /// but it is capped at a maximum interval.
    ///
    /// # Notes
    /// It currently uses a multiplier-based approach, and as such the growth
    /// will depend on the initial base reminder interval.
    ///
    /// # Parameters
    /// - `now`: The current time to compare against the last alert/reminder sent.
    /// - `reminder_interval`: The base interval to use for calculating when
    ///   the next reminder is due, which will be multiplied by a growth factor
    ///   based on the number of consecutive reminders already sent.
    ///
    /// # Returns
    /// - `true` if the next reminder is due based on the time since the last
    ///   reminder or last alert sent.
    /// - `false` if the next reminder is not yet due based on the time since
    ///   the last relevant event and the calculated next reminder interval.
    pub fn next_reminder_is_due(
        &self,
        now: &time::SystemTime,
        reminder_interval: &time::Duration,
    ) -> bool {
        let last_sent = match (self.last_reminder_sent, self.last_alert_sent) {
            (Some(last_reminder_sent), None) => {
                // A reminder has been recently sent so compare against that
                last_reminder_sent
            }
            (None, Some(last_alert_sent)) => {
                // No reminder has been sent yet but a normal alert has,
                // so compare against that
                last_alert_sent
            }
            (None, None) => {
                // Nothing has been sent yet
                return true;
            }
            _ => {
                // Any other combination is an error state and should never happen
                return true;
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
    /// frequently for notifications that have been unresolved for a long time,
    /// but it is capped at a maximum interval.
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
            return true;
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

    /// Handles the logic for when an alert or reminder fails to send.
    ///
    /// # Notes
    /// This method should be called whenever a send attempt fails, regardless of
    /// whether it was a new alert or a reminder, to ensure that the state
    /// is updated correctly for retries.
    ///
    /// # Parameters
    /// - `ctx`: The notification context to save for the failed alert or reminder.
    /// - `delta`: An optional delta representing the changes in peer status that
    ///   triggered the alert. If `None`, this indicates that the failure
    ///   was a reminder rather than an alert.
    pub fn on_failure(&mut self, ctx: &super::Context, delta: Option<&super::KeyDelta>) {
        self.last_failed_send = Some(ctx.now);
        self.num_consecutive_failures += 1;

        match &mut self.failed_ctx {
            Some(first_failed_ctx) => {
                first_failed_ctx.merge(ctx);
            }
            None => {
                let mut new_failed_ctx = ctx.clone();
                new_failed_ctx.has_failed = true;
                self.failed_ctx = Some(new_failed_ctx);
            }
        }

        match &mut self.failed_delta {
            Some(first_failed_delta) => {
                if let Some(delta) = delta {
                    first_failed_delta.merge(delta);
                }
            }
            None => {
                self.failed_delta = delta.cloned();
            }
        }
    }

    /// Handles the logic for when a reminder is successfully sent.
    ///
    /// # Parameters
    /// - `now`: The current time.
    pub fn on_successful_reminder(&mut self, now: &time::SystemTime) {
        self.last_alert_sent = None;
        self.last_reminder_sent = Some(*now);
        self.num_consecutive_reminders += 1;
    }

    /// Handles the logic for when an alert is successfully sent.
    ///
    /// # Parameters
    /// - `now`: The current time.
    pub fn on_successful_alert(&mut self, now: &time::SystemTime) {
        self.last_alert_sent = Some(*now);
        self.last_reminder_sent = None;
        self.num_consecutive_reminders = 0;
    }

    /// Handles the logic for when a retry attempt is successful.
    pub fn on_successful_retry(&mut self) {
        self.last_failed_send = None;
        self.num_consecutive_failures = 0;
    }
}
