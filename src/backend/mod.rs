//! Backend module for notification backends (e.g., Slack, Batsign).

mod batsign;
mod slack;
mod traits;

pub use batsign::BatsignBackend;
pub use slack::SlackBackend;
pub use traits::Backend;
