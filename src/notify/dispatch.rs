//! Handles the dispatching of notifications to all configured notifiers,
//! including sending notifications about peer status changes and sending
//! reminder notifications.

use crate::settings;

/// Terminal separator line used in logging output.
const SEP: &str = "--------------------";

/// Small helper that prints an `Option<String>` message if it is `Some` and the
/// `verbose` setting is enabled.
fn verbose_print_option(message: &Option<String>, settings: &settings::Settings) {
    if settings.verbose
        && let Some(msg) = message
    {
        println!("{SEP}\n{msg}\n{SEP}");
    }
}

/// Small helper that prints a message if the `verbose` setting is enabled.
fn verbose_print(message: &str, settings: &settings::Settings) {
    if settings.verbose {
        println!("{SEP}\n{message}\n{SEP}");
    }
}

/// Retries sending any notifications stored in notifiers.
pub fn retry_stored_notification(
    notifiers: &mut [Box<dyn super::NotificationSender>],
    settings: &settings::Settings,
) -> super::DispatchReport {
    let mut report = super::DispatchReport {
        total: notifiers.len() as u32,
        ..Default::default()
    };

    for n in notifiers.iter_mut() {
        let (result, message) = retry_single_notification(n, settings);

        match result {
            super::NotificationResult::DryRun => {
                verbose_print_option(&message, settings);
                report.successful += 1;
            }
            super::NotificationResult::Success => {
                verbose_print_option(&message, settings);
                report.successful += 1;
            }
            super::NotificationResult::Failure(_) => {
                verbose_print_option(&message, settings);
                report.failed += 1;
            }
            super::NotificationResult::Skipped => {
                verbose_print_option(&message, settings);
                report.skipped += 1;
            }
        }
    }

    report
}

/// Retries sending a single stored notification in one notifier.
/// The notification may be a reminder.
pub fn retry_single_notification(
    n: &mut Box<dyn super::NotificationSender>,
    settings: &settings::Settings,
) -> (super::NotificationResult, Option<String>) {
    match n.get_stored_notification() {
        // If it has a Context and a Delta, it is a notification
        // If it only has a Context, it is a reminder
        // If it has neither, it doesn't have a stored notification
        (Some(ctx), Some(delta)) => {
            match n.push_notification(&ctx, &delta) {
                (super::NotificationResult::DryRun, message) => {
                    println!("[{}] DRY RUN", n.name());
                    verbose_print(&message, settings);
                    n.clear_stored_notification(); // Notification sent so discard it
                    (super::NotificationResult::DryRun, Some(message))
                }
                (super::NotificationResult::Success, message) => {
                    println!("[{}] Notification sent successfully", n.name());
                    verbose_print(&message, settings);
                    n.clear_stored_notification(); // As above, discard it
                    (super::NotificationResult::Success, Some(message))
                }
                (super::NotificationResult::Failure(e), message) => {
                    eprintln!("[{}] Failed to send notification: {e}", n.name());
                    verbose_print(&message, settings);
                    (super::NotificationResult::Failure(e), Some(message))
                }
                _ => {
                    // Should never happen.
                    (super::NotificationResult::Skipped, None)
                }
            }
        }
        (Some(ctx), None) => match n.push_reminder(&ctx) {
            (super::NotificationResult::DryRun, message) => {
                println!("[{}] DRY RUN", n.name());
                verbose_print(&message, settings);
                n.clear_stored_notification(); // Reminder sent so discard it
                n.increment_num_consecutive_reminders();
                (super::NotificationResult::DryRun, Some(message))
            }
            (super::NotificationResult::Success, message) => {
                println!("[{}] Reminder sent successfully", n.name());
                verbose_print(&message, settings);
                n.clear_stored_notification(); // As above
                n.set_last_reminder_sent(Some(ctx.now));
                n.increment_num_consecutive_reminders();
                (super::NotificationResult::Success, Some(message))
            }
            (super::NotificationResult::Failure(e), message) => {
                eprintln!("[{}] Failed to send reminder: {e}", n.name());
                verbose_print(&message, settings);
                (super::NotificationResult::Failure(e), Some(message))
            }
            _ => {
                // Should never happen.
                (super::NotificationResult::Skipped, None)
            }
        },
        (None, _) => (super::NotificationResult::Skipped, None),
    }
}

/// Sends a notification via all notifiers.
pub fn send_notification(
    ctx: &super::Context,
    delta: &super::Delta,
    notifiers: &mut [Box<dyn super::NotificationSender>],
    settings: &settings::Settings,
) -> super::DispatchReport {
    let mut report = super::DispatchReport {
        total: notifiers.len() as u32,
        ..Default::default()
    };

    for n in notifiers.iter_mut() {
        let (result, _) = send_single_notifier_notification(ctx, delta, n, settings);

        match result {
            super::NotificationResult::DryRun => {
                report.successful += 1;
            }
            super::NotificationResult::Success => {
                report.successful += 1;
            }
            super::NotificationResult::Failure(_) => {
                report.failed += 1;
            }
            super::NotificationResult::Skipped => {
                report.skipped += 1;
            }
        }
    }

    report
}

/// Sends a single notification via one notifier.
fn send_single_notifier_notification(
    ctx: &super::Context,
    delta: &super::Delta,
    n: &mut Box<dyn super::NotificationSender>,
    settings: &settings::Settings,
) -> (super::NotificationResult, Option<String>) {
    // Time to send a new notification, so discard any stored ones
    n.clear_stored_notification();
    n.reset_num_consecutive_reminders();
    n.clear_last_reminder_sent();

    match n.push_notification(ctx, delta) {
        (super::NotificationResult::DryRun, message) => {
            println!("[{}] DRY RUN", n.name());
            verbose_print(&message, settings);
            (super::NotificationResult::DryRun, Some(message))
        }
        (super::NotificationResult::Success, message) => {
            println!("[{}] Notification sent successfully", n.name());
            verbose_print(&message, settings);
            (super::NotificationResult::Success, Some(message))
        }
        (super::NotificationResult::Failure(e), message) => {
            eprintln!("[{}] Failed to send notification: {e}", n.name());
            verbose_print(&message, settings);
            n.store_notification(ctx, Some(delta)); // Store the failure for retrying
            (super::NotificationResult::Failure(e), Some(message))
        }
        _ => {
            // Should never happen.
            (super::NotificationResult::Skipped, None)
        }
    }
}

/// Sends a single reminder notification via one notifier.
pub fn send_single_notifier_reminder(
    ctx: &super::Context,
    n: &mut Box<dyn super::NotificationSender>,
    settings: &settings::Settings,
) -> (super::NotificationResult, Option<String>) {
    // Time to send a new reminder, so discard any stored ones
    n.clear_stored_notification();

    match n.push_reminder(ctx) {
        (super::NotificationResult::DryRun, message) => {
            println!("[{}] DRY RUN", n.name());
            verbose_print(&message, settings);
            n.set_last_reminder_sent(Some(ctx.now));
            n.increment_num_consecutive_reminders();
            (super::NotificationResult::DryRun, Some(message))
        }
        (super::NotificationResult::Success, message) => {
            println!("[{}] Reminder sent successfully", n.name());
            verbose_print(&message, settings);
            n.set_last_reminder_sent(Some(ctx.now));
            n.increment_num_consecutive_reminders();
            (super::NotificationResult::Success, Some(message))
        }
        (super::NotificationResult::Failure(e), message) => {
            eprintln!("[{}] Failed to send reminder: {e}", n.name());
            verbose_print(&message, settings);
            n.store_notification(ctx, None);
            (super::NotificationResult::Failure(e), Some(message))
        }
        _ => {
            // Should never happen.
            (super::NotificationResult::Skipped, None)
        }
    }
}
