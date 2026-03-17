//! Defines the `NotificationSender` trait, implemented by types that can send
//! notifications about WireGuard peer status changes.

/// Trait for types that can send notifications about WireGuard peer status changes.
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
