//! Handles the dispatching of notifications to all configured notifiers,
//! including sending notifications about peer status changes and sending
//! reminder notifications.

use crate::settings;
use crate::utils;

/// Terminal separator line used in logging output.
const SEP: &str = "--------------------";

/// Small helper that prints a message if the `verbose` setting is enabled.
fn verbose_print(message: &str, settings: &settings::Settings) {
    if settings.verbose {
        println!("{SEP}\n{message}\n{SEP}");
    }
}

/// Retries sending any notifications stored in notifiers.
pub fn retry_stored_notification(
    notifiers: &mut [Box<dyn super::StatefulNotifier>],
    settings: &settings::Settings,
) -> super::DispatchReport {
    let mut report = super::DispatchReport {
        total: notifiers.len() as u32,
        ..Default::default()
    };

    for n in notifiers.iter_mut() {
        let result = retry_single_notification(n, settings);

        match result {
            super::NotificationResult::DryRun(message) => {
                verbose_print(&message, settings);
                report.successful += 1;
            }
            super::NotificationResult::Success(message) => {
                verbose_print(&message, settings);
                report.successful += 1;
            }
            super::NotificationResult::Failure(_, message) => {
                verbose_print(&message, settings);
                report.failed += 1;
            }
            super::NotificationResult::Skipped => {
                report.skipped += 1;
            }
        }
    }

    report
}

/// Retries sending a single stored notification in one notifier.
/// The notification may be a reminder.
pub fn retry_single_notification(
    n: &mut Box<dyn super::StatefulNotifier>,
    settings: &settings::Settings,
) -> super::NotificationResult {
    match n.state_mut().take_stored_notification() {
        // Taken; stored notification is now None
        Some(super::StoredNotification::Notification(ctx, delta)) => {
            match n.push_notification(&ctx, &delta) {
                super::NotificationResult::DryRun(message) => {
                    println!("[{}] [{}] DRY RUN", utils::timestamp_now(), n.name());
                    verbose_print(&message, settings);
                    super::NotificationResult::DryRun(message)
                }
                super::NotificationResult::Success(message) => {
                    println!(
                        "[{}] [{}] Notification sent successfully",
                        utils::timestamp_now(),
                        n.name()
                    );
                    verbose_print(&message, settings);
                    super::NotificationResult::Success(message)
                }
                super::NotificationResult::Failure(e, message) => {
                    eprintln!(
                        "[{}] [{}] Failed to send notification: {e}",
                        utils::timestamp_now(),
                        n.name()
                    );

                    // Put the notification back for later retries
                    n.state_mut().store_notification(&ctx, Some(&delta));
                    super::NotificationResult::Failure(e, message)
                }
                super::NotificationResult::Skipped => {
                    // push_notification does not return Skipped, so this can never happen.
                    super::NotificationResult::Skipped
                }
            }
        }
        Some(super::StoredNotification::Reminder(ctx)) => match n.push_reminder(&ctx) {
            super::NotificationResult::DryRun(message) => {
                println!("[{}] [{}] DRY RUN", utils::timestamp_now(), n.name());
                verbose_print(&message, settings);
                n.state_mut().increment_num_consecutive_reminders();
                super::NotificationResult::DryRun(message)
            }
            super::NotificationResult::Success(message) => {
                println!(
                    "[{}] [{}] Reminder sent successfully",
                    utils::timestamp_now(),
                    n.name()
                );
                verbose_print(&message, settings);
                n.state_mut().set_last_reminder_sent(Some(ctx.now));
                n.state_mut().increment_num_consecutive_reminders();
                super::NotificationResult::Success(message)
            }
            super::NotificationResult::Failure(e, message) => {
                eprintln!(
                    "[{}] [{}] Failed to send reminder: {e}",
                    utils::timestamp_now(),
                    n.name()
                );

                // Put the notification back for later retries
                n.state_mut().store_notification(&ctx, None);
                super::NotificationResult::Failure(e, message)
            }
            super::NotificationResult::Skipped => {
                // push_reminder does not return Skipped, so this can never happen.
                super::NotificationResult::Skipped
            }
        },
        None => {
            // No notification stored
            super::NotificationResult::Skipped
        }
    }
}

/// Sends a notification via all notifiers.
pub fn send_notification(
    ctx: &super::Context,
    delta: &super::Delta,
    notifiers: &mut [Box<dyn super::StatefulNotifier>],
    settings: &settings::Settings,
) -> super::DispatchReport {
    let mut report = super::DispatchReport {
        total: notifiers.len() as u32,
        ..Default::default()
    };

    for n in notifiers.iter_mut() {
        match send_single_notifier_notification(ctx, delta, n, settings) {
            super::NotificationResult::DryRun(_) => {
                report.successful += 1;
            }
            super::NotificationResult::Success(_) => {
                report.successful += 1;
            }
            super::NotificationResult::Failure(_, _) => {
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
    n: &mut Box<dyn super::StatefulNotifier>,
    settings: &settings::Settings,
) -> super::NotificationResult {
    // Time to send a new notification, so discard any stored ones
    n.state_mut().clear_stored_notification();
    n.state_mut().reset_num_consecutive_reminders();
    n.state_mut().clear_last_reminder_sent();

    match n.push_notification(ctx, delta) {
        super::NotificationResult::DryRun(message) => {
            println!("[{}] [{}] DRY RUN", utils::timestamp_now(), n.name());
            verbose_print(&message, settings);
            super::NotificationResult::DryRun(message)
        }
        super::NotificationResult::Success(message) => {
            println!(
                "[{}] [{}] Notification sent successfully",
                utils::timestamp_now(),
                n.name()
            );
            verbose_print(&message, settings);
            super::NotificationResult::Success(message)
        }
        super::NotificationResult::Failure(e, message) => {
            eprintln!(
                "[{}] [{}] Failed to send notification: {e}",
                utils::timestamp_now(),
                n.name()
            );
            n.state_mut().store_notification(ctx, Some(delta)); // Store the failure for retrying
            super::NotificationResult::Failure(e, message)
        }
        super::NotificationResult::Skipped => {
            // push_notification does not return Skipped, so this can never happen.
            super::NotificationResult::Skipped
        }
    }
}

/// Sends a single reminder notification via one notifier.
pub fn send_single_notifier_reminder(
    ctx: &super::Context,
    n: &mut Box<dyn super::StatefulNotifier>,
    settings: &settings::Settings,
) -> super::NotificationResult {
    // Time to send a new reminder, so discard any stored ones
    n.state_mut().clear_stored_notification();

    match n.push_reminder(ctx) {
        super::NotificationResult::DryRun(message) => {
            println!("[{}] [{}] DRY RUN", utils::timestamp_now(), n.name());
            verbose_print(&message, settings);
            n.state_mut().set_last_reminder_sent(Some(ctx.now));
            n.state_mut().increment_num_consecutive_reminders();
            super::NotificationResult::DryRun(message)
        }
        super::NotificationResult::Success(message) => {
            println!(
                "[{}] [{}] Reminder sent successfully",
                utils::timestamp_now(),
                n.name()
            );
            verbose_print(&message, settings);
            n.state_mut().set_last_reminder_sent(Some(ctx.now));
            n.state_mut().increment_num_consecutive_reminders();
            super::NotificationResult::Success(message)
        }
        super::NotificationResult::Failure(e, message) => {
            eprintln!(
                "[{}] [{}] Failed to send reminder: {e}",
                utils::timestamp_now(),
                n.name()
            );
            n.state_mut().store_notification(ctx, None);
            super::NotificationResult::Failure(e, message)
        }
        super::NotificationResult::Skipped => {
            // push_reminder does not return Skipped, so this can never happen.
            super::NotificationResult::Skipped
        }
    }
}
