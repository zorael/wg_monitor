//! Types related to notification reporting.

/// Struct representing a report of the results of a notification dispatch,
/// including counts of total, successful, failed, no message,
/// and skipped notifications.
///
/// Used to summarize the outcomes of a batch of notification attempts
/// for terminal output.
#[allow(unused)]
#[derive(Default, Debug)]
pub struct DispatchReport {
    /// Total number of notification attempts made.
    pub total: u32,

    /// Number of notifications that were successfully sent.
    pub successful: u32,

    /// Number of notifications that failed to send.
    pub failed: u32,

    /// Number of notifications that were skipped due to the rendered message
    /// ending up empty.
    pub no_message: u32,

    /// Number of notifications that were skipped.
    pub skipped: u32,
}
