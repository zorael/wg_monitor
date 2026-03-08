//! Handles the dispatching of notifications to all configured notifiers,
//! including sending notifications about peer status changes and sending
//! reminder notifications.

use std::time;

use crate::settings;
use crate::utils;

/// Small helper that prints a message if the `verbose` setting is enabled.
fn verbose_print(message: &str, settings: &settings::Settings) {
    const SEP: &str = "--------------------";

    if settings.verbose {
        println!("{SEP}\n{message}\n{SEP}");
    }
}

/// Retries sending any notifications stored in notifiers.
pub fn retry_pending_notifications(
    notifiers: &mut [Box<dyn super::StatefulNotifier>],
    settings: &settings::Settings,
) -> super::DispatchReport {
    let mut report = super::DispatchReport {
        total: notifiers.len() as u32,
        ..Default::default()
    };

    for n in notifiers.iter_mut() {
        match retry_notifier(n) {
            super::NotificationResult::DryRun(message) => {
                println!(
                    "[{}] [{}] DRY RUN; not sent",
                    utils::timestamp_now(),
                    n.name()
                );

                verbose_print(&message, settings);
                report.successful += 1;
            }
            super::NotificationResult::Success(message) => {
                println!(
                    "[{}] [{}] Notification sent successfully",
                    utils::timestamp_now(),
                    n.name()
                );

                verbose_print(&message, settings);
                report.successful += 1;
            }
            super::NotificationResult::Failure(e, message) => {
                eprintln!(
                    "[{}] [{}] Failed to send notification: {e}",
                    utils::timestamp_now(),
                    n.name()
                );

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

/// Retries sending a single pending notification in one notifier.
/// The notification may be a reminder.
fn retry_notifier(n: &mut Box<dyn super::StatefulNotifier>) -> super::NotificationResult {
    let now = time::SystemTime::now();

    if !n.state().next_retry_is_due(&now) {
        // Not yet time
        return super::NotificationResult::Skipped;
    }

    match n.state_mut().pending.take() {
        // Taken; pending notification is now None, so failure cases must put it back
        Some(super::StoredNotification::Notification(ctx, delta)) => {
            let result = n.push_notification(&ctx, &delta);

            match &result {
                super::NotificationResult::DryRun(_) => {
                    n.state_mut().on_successful_notification();
                }
                super::NotificationResult::Success(_) => {
                    n.state_mut().on_successful_notification();
                }
                super::NotificationResult::Failure(_, _) => {
                    n.state_mut().on_failure(&ctx, Some(&delta), &now);
                }
                super::NotificationResult::Skipped => {
                    // push_notification does not return Skipped, so this can never happen.
                }
            }

            result
        }
        Some(super::StoredNotification::Reminder(ctx)) => {
            let result = n.push_reminder(&ctx);

            match &result {
                super::NotificationResult::DryRun(_) => {
                    n.state_mut().on_successful_reminder(&now);
                }
                super::NotificationResult::Success(_) => {
                    n.state_mut().on_successful_reminder(&now);
                }
                super::NotificationResult::Failure(_, _) => {
                    n.state_mut().on_failure(&ctx, None, &now);
                }
                super::NotificationResult::Skipped => {
                    // push_reminder does not return Skipped, so this can never happen.
                }
            }

            result
        }
        None => {
            // No notification pending
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
        match send_notification_via_notifier(ctx, delta, n) {
            super::NotificationResult::DryRun(message) => {
                println!(
                    "[{}] [{}] DRY RUN; not sent",
                    utils::timestamp_now(),
                    n.name()
                );

                verbose_print(&message, settings);
                report.successful += 1;
            }
            super::NotificationResult::Success(message) => {
                println!(
                    "[{}] [{}] Notification sent successfully",
                    utils::timestamp_now(),
                    n.name()
                );

                verbose_print(&message, settings);
                report.successful += 1;
            }
            super::NotificationResult::Failure(e, message) => {
                eprintln!(
                    "[{}] [{}] Failed to send notification: {e}",
                    utils::timestamp_now(),
                    n.name()
                );

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

/// Sends a single notification via one notifier.
fn send_notification_via_notifier(
    ctx: &super::Context,
    delta: &super::Delta,
    n: &mut Box<dyn super::StatefulNotifier>,
) -> super::NotificationResult {
    // Time to send a new notification, so discard anything old
    n.state_mut().reset();

    let result = n.push_notification(ctx, delta);

    match &result {
        super::NotificationResult::DryRun(_) => {}
        super::NotificationResult::Success(_) => {}
        super::NotificationResult::Failure(_, _) => {
            n.state_mut().on_failure(ctx, Some(delta), &ctx.now);
        }
        super::NotificationResult::Skipped => {}
    }

    result
}

/// Sends reminders via all notifiers.
pub fn send_reminders(
    ctx: &super::Context,
    notifiers: &mut [Box<dyn super::StatefulNotifier>],
    settings: &settings::Settings,
) -> super::DispatchReport {
    let mut report = super::DispatchReport {
        total: notifiers.len() as u32,
        ..Default::default()
    };

    for n in notifiers.iter_mut() {
        match send_reminder_via_notifier(ctx, n) {
            super::NotificationResult::DryRun(message) => {
                println!(
                    "[{}] [{}] DRY RUN; not sent",
                    utils::timestamp_now(),
                    n.name()
                );

                verbose_print(&message, settings);
                report.successful += 1;
            }
            super::NotificationResult::Success(message) => {
                println!(
                    "[{}] [{}] Reminder sent successfully",
                    utils::timestamp_now(),
                    n.name()
                );

                verbose_print(&message, settings);
                report.successful += 1;
            }
            super::NotificationResult::Failure(e, message) => {
                eprintln!(
                    "[{}] [{}] Failed to send reminder: {e}",
                    utils::timestamp_now(),
                    n.name()
                );

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

/// Sends a single reminder notification via one notifier.
fn send_reminder_via_notifier(
    ctx: &super::Context,
    n: &mut Box<dyn super::StatefulNotifier>,
) -> super::NotificationResult {
    if !n.state().next_reminder_is_due(&ctx.now) {
        // Not yet time to send the next reminder
        return super::NotificationResult::Skipped;
    }

    // Time to send a new reminder, so discard anything old
    n.state_mut().reset();

    let result = n.push_reminder(ctx);

    match &result {
        super::NotificationResult::DryRun(_) => {
            n.state_mut().on_successful_reminder(&ctx.now);
        }
        super::NotificationResult::Success(_) => {
            n.state_mut().on_successful_reminder(&ctx.now);
        }
        super::NotificationResult::Failure(_, _) => {
            n.state_mut().on_failure(ctx, None, &ctx.now);
        }
        super::NotificationResult::Skipped => {}
    }

    result
}
