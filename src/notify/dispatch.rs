//! Handles the dispatching of notifications to all configured notifiers,
//! including sending notifications about peer status changes and sending
//! reminder notifications.

/// Terminal separator line used in logging output.
const SEP: &str = "--------------------";

/// Sends a notification via all notifiers, returning `true` if all notifications
/// were sent successfully, or `false` if any failed.
pub fn send_notification(
    notifiers: &mut [Box<dyn super::NotificationSender>],
    ctx: &super::Context,
    delta: &super::Delta,
    verbose: bool,
) -> bool {
    let mut success = true;

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
            }
            (super::NotificationResult::Success, message) => {
                println!("[{}] Notification sent successfully", n.name());
                verbose_print(&message);
            }
            (super::NotificationResult::Failure(e), message) => {
                eprintln!("[{}] Failed to send notification: {e}", n.name());
                verbose_print(&message);
                success = false;
            }
        }
    }

    success
}

/// Sends a reminder notification via all notifiers, returning `true` if all notifications
/// were sent successfully, or `false` if any failed.
pub fn send_reminder(
    notifiers: &mut [Box<dyn super::NotificationSender>],
    ctx: &super::Context,
    verbose: bool,
) -> bool {
    let mut success = true;

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
            }
            (super::NotificationResult::Success, message) => {
                println!("[{}] Reminder sent successfully", n.name());
                verbose_print(&message);
            }
            (super::NotificationResult::Failure(e), message) => {
                eprintln!("[{}] Failed to send reminder: {e}", n.name());
                verbose_print(&message);
                success = false;
            }
        }
    }

    success
}
