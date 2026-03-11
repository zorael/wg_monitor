//! Backend traits for notification backends (e.g., Slack, Batsign).

use crate::notify;

/// The `Backend` trait, which is implemented by all notification backends
/// (e.g., Slack, Batsign).
pub trait Backend {
    /// Returns the unique identifier of the backend instance.
    #[allow(dead_code)]
    fn id(&self) -> usize;

    /// Returns the name of the instance of the backend, which is used for
    /// logging and identification purposes.
    fn name(&self) -> &str;

    /// Builds the message to be sent based on the notification context and the
    /// delta expressing the changes since the last notification.
    fn build_message(&self, ctx: &notify::Context, delta: &notify::Delta) -> String;

    /// Builds the reminder message to be sent based on the notification context.
    fn build_reminder(&self, ctx: &notify::Context) -> String;

    /// Delivers the already-built message using backend-owned methods.
    fn emit(
        &mut self,
        ctx: &notify::Context,
        delta: Option<&notify::Delta>,
        message: &str,
    ) -> Result<Option<String>, String>;
}
