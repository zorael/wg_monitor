//! Handles the dispatching of notifications to all configured notifiers,
//! including sending notifications about peer status changes and sending
//! reminder notifications.

/// Terminal separator line used in logging output.
const SEP: &str = "--------------------";

/// Sends a notification via all notifiers.
pub fn send_notification(
    notifiers: &mut [Box<dyn super::NotificationSender>],
    ctx: &super::Context,
    delta: &super::Delta,
    verbose: bool,
) -> super::DispatchReport {
    let mut report = super::DispatchReport::default();
    report.total = notifiers.len() as u32;

    let verbose_print = |message: &str| {
        if verbose {
            println!("{SEP}\n{message}\n{SEP}");
        }
    };

    for n in notifiers.iter_mut() {
        match n.push_notification(ctx, delta) {
            (super::NotificationResult::DryRun, message) => {
                println!("[{}] DRY RUN", n.name());
                verbose_print(&message);
                report.skipped += 1;
            }
            (super::NotificationResult::Success, message) => {
                println!("[{}] Notification sent successfully", n.name());
                verbose_print(&message);
                report.successful += 1;
            }
            (super::NotificationResult::Failure(e), message) => {
                eprintln!("[{}] Failed to send notification: {e}", n.name());
                verbose_print(&message);
                report.failed += 1;
            }
        }
    }

    report
}

/// Sends a reminder notification via all notifiers.
pub fn send_reminder(
    notifiers: &mut [Box<dyn super::NotificationSender>],
    ctx: &super::Context,
    verbose: bool,
) -> super::DispatchReport {
    let mut report = super::DispatchReport::default();
    report.total = notifiers.len() as u32;

    let verbose_print = |message: &str| {
        if verbose {
            println!("{SEP}\n{message}\n{SEP}");
        }
    };

    for n in notifiers.iter_mut() {
        match n.push_reminder(ctx) {
            (super::NotificationResult::DryRun, message) => {
                println!("[{}] DRY RUN", n.name());
                verbose_print(&message);
                report.skipped += 1;
            }
            (super::NotificationResult::Success, message) => {
                println!("[{}] Reminder sent successfully", n.name());
                verbose_print(&message);
                report.successful += 1;
            }
            (super::NotificationResult::Failure(e), message) => {
                eprintln!("[{}] Failed to send reminder: {e}", n.name());
                verbose_print(&message);
                report.failed += 1;
            }
        }
    }

    report
}
