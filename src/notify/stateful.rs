//! Traits and implementations for stateful notifiers, which are notification
//! senders that also carry a `NotifierState` to manage pending notifications,
//! reminder timing, and failure tracking.

use crate::backend;

/// Supertrait that combines the functionality of a `NotificationSender` with
/// the ability to carry and manage a `NotifierState` for tracking pending
/// notifications, reminder timing, and failure tracking.
pub trait StatefulNotifier: super::NotificationSender + StateCarrier {}

/// Blanket implementation of `StatefulNotifier` for any type that implements
/// both `NotificationSender` and `StateCarrier`.
///
/// This allows any type that implements both traits to automatically be
/// considered a `StatefulNotifier`, which can be useful for writing generic
/// code that operates on stateful notifiers without needing to specify the
/// exact type of notifier being used.
impl<T: super::NotificationSender + StateCarrier> StatefulNotifier for T {}

/// Trait for types that can carry a `NotifierState`, which is used to track
/// pending notifications, reminder timing, and failure tracking.
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
