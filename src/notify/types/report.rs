//! Types related to notification reporting.

/// Report struct returned from functions dispatching notifications.
#[allow(unused)]
#[derive(Default, Debug)]
pub struct DispatchReport {
    /// Total number of notifications attempted to be sent.
    pub total: u32,

    /// Number of notifications successfully sent.
    pub successful: u32,

    /// Number of notifications that failed to send.
    pub failed: u32,

    /// Number of notifications that were skipped due to missing message content.
    pub no_message: u32,

    /// Number of notifications that were skipped.
    pub skipped: u32,
}
