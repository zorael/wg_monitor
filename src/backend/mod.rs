//! Backends for sending notifications.
//!
//! Backends are responsible for composing messages based on the notification
//! context and delta, and for sending the notifications through the appropriate
//! channels.
//!
//! Each backend implements the `Backend` trait, which defines the required
//! methods for composing and sending notifications.
//!
//! - The `BatsignBackend` sends notifications to the free Batsign service by
//!   making HTTP POST requests to a unique URL.
//! - The `SlackBackend` sends notifications to a Slack channel via webhooks.
//! - The `CommandBackend` executes a specified command to send notifications,
//!   allowing for custom handling of notifications through scripts or other executables.

mod api;
mod batsign;
mod command;
mod slack;

pub use api::Backend;
pub use batsign::BatsignBackend;
pub use command::CommandBackend;
pub use slack::SlackBackend;
