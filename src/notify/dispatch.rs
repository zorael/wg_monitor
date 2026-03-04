//! Handles the dispatching of notifications to all configured notifiers,
//! including sending notifications about peer status changes and sending
//! reminder notifications.

/// Terminal separator line used in logging output.
const SEP: &str = "--------------------";

/// Sends a notification via all notifiers, returning true if all notifications
/// were sent successfully, or false if any failed.
pub fn send_notification(
    notifiers: &mut [Box<dyn super::NotificationSender>],
    ctx: &super::Context,
    delta: &super::Delta,
) -> bool {
    let mut success = true;

    for n in notifiers.iter_mut() {
        match n.push_notification(ctx, delta) {
            super::NotificationResult::DryRun(message) => {
                println!(
                    "[{}] DRY RUN, would have sent:\n{SEP}\n{message}\n{SEP}",
                    n.name()
                );
            }
            super::NotificationResult::Success => {
                println!("[{}] Notification sent successfully", n.name());
            }
            super::NotificationResult::Failure(message) => {
                eprintln!(
                    "[{}] Failed to send notification:\n{SEP}\n{message}\n{SEP}",
                    n.name()
                );
                success = false;
            }
        }
    }

    success
}

/// Sends a reminder notification via all notifiers, returning true if all notifications
/// were sent successfully, or false if any failed.
pub fn send_reminder(
    notifiers: &mut [Box<dyn super::NotificationSender>],
    ctx: &super::Context,
) -> bool {
    let mut success = true;

    for n in notifiers.iter_mut() {
        match n.push_reminder(ctx) {
            super::NotificationResult::DryRun(message) => {
                println!(
                    "[{}] DRY RUN, would have sent reminder:\n{SEP}\n{message}\n{SEP}",
                    n.name()
                );
            }
            super::NotificationResult::Success => {
                println!("[{}] Reminder sent successfully", n.name());
            }
            super::NotificationResult::Failure(message) => {
                eprintln!(
                    "[{}] Failed to send reminder:\n{SEP}\n{message}\n{SEP}",
                    n.name()
                );
                success = false;
            }
        }
    }

    success
}
