//! Defines the `Backend` trait, which specifies the interface for all
//! notification backends.

use crate::notify;

/// The `Backend` trait defines the interface that all notification backends must
/// implement. This includes methods for composing messages and reminders based on
/// the notification context and delta, as well as emitting the notifications.
///
/// Backends are responsible for formatting messages according to their specific
/// requirements (such as JSON for `SlackBackend`) and for sending the notifications through
/// the appropriate channels (like HTTP requests for `SlackBackend`, command execution
/// for `CommandBackend`).
pub trait Backend {
    /// Returns the unique identifier of the backend instance.
    fn id(&self) -> usize;

    /// Returns the name of the backend instance.
    ///
    /// The name is used for logging and identification purposes, and may include
    /// additional information such as identifiers unique to each instance.
    fn name(&self) -> &str;

    /// Composes an alert message based on the notification context and
    /// the delta expressing the changes since the last check.
    ///
    /// # Parameters
    /// - `ctx`: Current notification context.
    /// - `delta`: The state change that triggered the alert.
    ///
    /// # Returns
    /// - `Some(String)` if a message should be sent
    /// - `None` if the composed message was empty, in which case nothing
    ///   will be sent.
    fn compose_alert(&self, ctx: &notify::Context, delta: &notify::KeyDelta) -> Option<String>;

    /// Composes a reminder message based on the notification context.
    ///
    /// Reminders differ from alerts in that they do not include
    /// a delta, since they are not triggered by a new state change.
    ///
    /// # Parameters
    /// - `ctx`: Current notification context.
    ///
    /// # Returns
    /// - `Some(String)` if a message to send was composed.
    /// - `None` if the composed message was empty, in which case nothing
    ///   will be sent.
    fn compose_reminder(&self, ctx: &notify::Context) -> Option<String>;

    /// Sends a composed notification message through this backend.
    ///
    /// `delta`, if present, describes the state change that triggered the notification.
    /// A value of `None` indicates that `message` is a reminder rather than
    /// a new alert.
    ///
    /// # Parameters
    /// - `ctx`: Current notification context.
    /// - `delta`: The triggering state change, or `None` for a reminder.
    /// - `message`: The already-composed message to send.
    ///
    /// # Returns
    /// - `Ok(None)` if the message was sent successfully.
    /// - `Ok(Some(String))` if a message was sent and the backend produced
    ///   informational output.
    /// - `Err(String)` if the send attempt failed.
    fn emit(
        &mut self,
        ctx: &notify::Context,
        delta: Option<&notify::KeyDelta>,
        message: &str,
    ) -> Result<Option<String>, String>;

    /// Performs a sanity check on the backend's configuration.
    ///
    /// What this does is implementation-defined.
    ///
    /// # Returns
    /// - `Ok(())` if the sanity check passed without any issues.
    /// - `Err(Vec<String>)` if there were issues found during the sanity check,
    ///   containing a vector of descriptive error messages for each issue found.
    fn sanity_check(&self) -> Result<(), Vec<String>>;
}
