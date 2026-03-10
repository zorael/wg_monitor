//! Defines the `StatefulNotifier` trait, which extends `NotificationSender` by
//! including a `NotifierState` for managing pending notifications, reminder
//! timing, and failure tracking.

use crate::backend;

/// A `StatefulNotifier` is a `NotificationSender` that also carries a `NotifierState`,
/// allowing it to manage pending notifications, reminder timing, and failure tracking.
pub trait StatefulNotifier: super::NotificationSender + StateCarrier {}

/// Blanket implementation of `StatefulNotifier` for any type that implements both
/// `NotificationSender` and `StateCarrier`.
impl<T: super::NotificationSender + StateCarrier> StatefulNotifier for T {}

/// Trait for types that carry a `NotifierState`, allowing access to the state
/// for managing pending notifications, reminder timing, and failure tracking.
pub trait StateCarrier {
    /// Returns a reference to the `NotifierState`.
    fn state(&self) -> &super::NotifierState;

    /// Returns a mutable reference to the `NotifierState`.
    fn state_mut(&mut self) -> &mut super::NotifierState;
}

impl<B: backend::Backend> StateCarrier for super::Notifier<B> {
    /// Returns a reference to the `NotifierState`.
    fn state(&self) -> &super::NotifierState {
        &self.state
    }

    /// Returns a mutable reference to the `NotifierState`.
    fn state_mut(&mut self) -> &mut super::NotifierState {
        &mut self.state
    }
}
