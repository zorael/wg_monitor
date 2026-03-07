//! Backend traits for notification backends (e.g., Slack, Batsign).

use crate::notify;

/// The `Backend` trait, which is implemented by all notification backends
/// (e.g., Slack, Batsign).
pub trait Backend {
    /// Returns the name of the instance of the backend, which is used for
    /// logging and identification purposes.
    fn name(&mut self) -> &str;

    /// Builds the message to be sent based on the notification context and the
    /// delta expressing the changes since the last notification.
    fn build_message(&self, ctx: &notify::Context, delta: &notify::Delta) -> String;

    /// Builds the reminder message to be sent based on the notification context.
    fn build_reminder(&self, ctx: &notify::Context) -> String;

    /// Delivers the already-built message using backend-owned methods.
    fn send(&mut self, message: &str) -> Result<(), String>;
}
