//! Backend module for notification backends (e.g., Slack, Batsign).

mod api;
mod batsign;
mod command;
mod slack;

pub use api::Backend;
pub use batsign::BatsignBackend;
pub use command::CommandBackend;
pub use slack::SlackBackend;
