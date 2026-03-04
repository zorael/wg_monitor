//! FIXME

use crate::notify;

/// Defines the `Backend` trait, which is implemented by all notification backends (e.g., Slack, Batsign).
pub trait Backend {
    /// Returns the name of the backend, which is used for logging and identification purposes.
    fn name(&self) -> String;

    /// Builds the message to be sent based on the notification context and delta.
    fn build_message(&self, ctx: &notify::Context, delta: &notify::Delta) -> String;

    /// Builds the reminder message to be sent based on the notification context.
    fn build_reminder(&self, ctx: &notify::Context) -> String;

    /// Deliver the already-built message using backend-owned configuration.
    fn send(&mut self, message: &str) -> Result<(), String>;
}
