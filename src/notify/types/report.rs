//! Types related to notification reporting.

#[allow(unused)]
#[derive(Default, Debug)]
pub struct DispatchReport {
    pub total: u32,
    pub successful: u32,
    pub failed: u32,
    pub skipped: u32,
}
