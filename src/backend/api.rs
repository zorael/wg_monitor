//! Defines the `Backend` trait, which specifies the interface for all
//! notification backends.
//!
//! This module also re-exports the specific backend implementations
//! (e.g., `SlackBackend`, `BatsignBackend`, `CommandBackend`) so that they
//! can be easily used by other parts of the application.

use crate::notify;

/// The `Backend` trait defines the interface that all notification backends must
/// implement. This includes methods for composing messages and reminders based on
/// the notification context and delta, as well as emitting the notifications.
///
/// Backends are responsible for formatting messages according to their specific
/// requirements (e.g., JSON for Slack) and for sending the notifications through
/// the appropriate channels (e.g., HTTP requests for Slack, command execution
/// for CommandBackend).
pub trait Backend {
    /// Returns the unique identifier of the backend instance.
    #[allow(dead_code)]
    fn id(&self) -> usize;

    /// Returns the name of the backend instance.
    ///
    /// The name is used for logging and identification purposes, and may include
    /// additional information such as unique identifiers.
    fn name(&self) -> &str;

    /// Composes a notification message based on the notification context and
    /// the delta expressing the changes since the last notification.
    ///
    /// # Parameters
    /// - `ctx`: Current notification context.
    /// - `delta`: The state change that triggered the notification.
    ///
    /// # Returns
    /// - `Some(message)` if a message should be sent
    /// - `None` if the composed message was empty, which typically means no
    ///   message should be sent.
    fn compose_message(&self, ctx: &notify::Context, delta: &notify::Delta) -> Option<String>;

    /// Composes a reminder message based on the notification context.
    ///
    /// # Parameters
    /// - `ctx`: Current notification context.
    ///
    /// # Returns
    /// - `Some(message)` if a message to send was composed.
    /// - `None` if an empty message was composed, typically meaning no message
    ///   should be sent.
    fn compose_reminder(&self, ctx: &notify::Context) -> Option<String>;

    /// Sends a composed notification message through this backend.
    ///
    /// `delta` describes the state change that triggered the notification.
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
    /// - `Ok(Some(output))` if a message was sent and the backend produced
    ///   informational output.
    /// - `Err(error)` if the send attempt failed.
    fn emit(
        &mut self,
        ctx: &notify::Context,
        delta: Option<&notify::Delta>,
        message: &str,
    ) -> Result<Option<String>, String>;
}
