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
pub fn retry_stored_notifications(
    notifiers: &mut [Box<dyn super::StatefulNotifier>],
    settings: &settings::Settings,
) -> super::DispatchReport {
    let mut report = super::DispatchReport {
        total: notifiers.len() as u32,
        ..Default::default()
    };

    for n in notifiers.iter_mut() {
        match retry_single_notification(n, settings) {
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

/// Retries sending a single stored notification in one notifier.
/// The notification may be a reminder.
pub fn retry_single_notification(
    n: &mut Box<dyn super::StatefulNotifier>,
    settings: &settings::Settings,
) -> super::NotificationResult {
    let now = time::SystemTime::now();

    if !n.state().next_retry_is_due(&now) {
        // Not yet time
        return super::NotificationResult::Skipped;
    }

    match n.state_mut().stored_notification.take() {
        // Taken; stored notification is now None
        Some(super::StoredNotification::Notification(ctx, delta)) => {
            match n.push_notification(&ctx, &delta) {
                super::NotificationResult::DryRun(message) => {
                    println!(
                        "[{}] [{}] DRY RUN; not sent",
                        utils::timestamp_now(),
                        n.name()
                    );

                    verbose_print(&message, settings);
                    n.state_mut().reset();
                    super::NotificationResult::DryRun(message)
                }
                super::NotificationResult::Success(message) => {
                    println!(
                        "[{}] [{}] Notification sent successfully",
                        utils::timestamp_now(),
                        n.name()
                    );

                    verbose_print(&message, settings);
                    n.state_mut().reset();
                    super::NotificationResult::Success(message)
                }
                super::NotificationResult::Failure(e, message) => {
                    eprintln!(
                        "[{}] [{}] Failed to send notification: {e}",
                        utils::timestamp_now(),
                        n.name()
                    );

                    verbose_print(&message, settings);

                    // Put the notification back for later retries
                    n.state_mut().store_notification(&ctx, Some(&delta));
                    n.state_mut().last_failed_send = Some(now);
                    n.state_mut().num_consecutive_failures += 1;
                    super::NotificationResult::Failure(e, message)
                }
                super::NotificationResult::Skipped => {
                    // push_notification does not return Skipped, so this can never happen.
                    super::NotificationResult::Skipped
                }
            }
        }
        Some(super::StoredNotification::Reminder(ctx)) => {
            match n.push_reminder(&ctx) {
                super::NotificationResult::DryRun(message) => {
                    println!(
                        "[{}] [{}] DRY RUN; not sent",
                        utils::timestamp_now(),
                        n.name()
                    );
                    verbose_print(&message, settings);
                    n.state_mut().last_reminder_sent = Some(ctx.now);
                    n.state_mut().num_consecutive_reminders += 1;
                    n.state_mut().num_consecutive_failures = 0;
                    n.state_mut().last_failed_send = None;
                    super::NotificationResult::DryRun(message)
                }
                super::NotificationResult::Success(message) => {
                    println!(
                        "[{}] [{}] Reminder sent successfully",
                        utils::timestamp_now(),
                        n.name()
                    );

                    verbose_print(&message, settings);
                    n.state_mut().last_reminder_sent = Some(ctx.now);
                    n.state_mut().num_consecutive_reminders += 1;
                    n.state_mut().num_consecutive_failures = 0;
                    n.state_mut().last_failed_send = None;
                    super::NotificationResult::Success(message)
                }
                super::NotificationResult::Failure(e, message) => {
                    eprintln!(
                        "[{}] [{}] Failed to send reminder: {e}",
                        utils::timestamp_now(),
                        n.name()
                    );

                    verbose_print(&message, settings);

                    // Put the notification back for later retries
                    n.state_mut().store_notification(&ctx, None);
                    n.state_mut().last_failed_send = Some(now);
                    n.state_mut().num_consecutive_failures += 1;
                    super::NotificationResult::Failure(e, message)
                }
                super::NotificationResult::Skipped => {
                    // push_reminder does not return Skipped, so this can never happen.
                    super::NotificationResult::Skipped
                }
            }
        }
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
        match send_notification_via_notifier(ctx, delta, n, settings) {
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
fn send_notification_via_notifier(
    ctx: &super::Context,
    delta: &super::Delta,
    n: &mut Box<dyn super::StatefulNotifier>,
    settings: &settings::Settings,
) -> super::NotificationResult {
    // Time to send a new notification, so discard anything old
    n.state_mut().reset();

    match n.push_notification(ctx, delta) {
        super::NotificationResult::DryRun(message) => {
            println!(
                "[{}] [{}] DRY RUN; not sent",
                utils::timestamp_now(),
                n.name()
            );

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

            verbose_print(&message, settings);
            n.state_mut().store_notification(ctx, Some(delta)); // Store the failure for retrying
            n.state_mut().num_consecutive_failures += 1;
            super::NotificationResult::Failure(e, message)
        }
        super::NotificationResult::Skipped => {
            // push_notification does not return Skipped, so this can never happen.
            super::NotificationResult::Skipped
        }
    }
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
        match send_reminder_via_notifier(ctx, n, settings) {
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

/// Sends a single reminder notification via one notifier.
pub fn send_reminder_via_notifier(
    ctx: &super::Context,
    n: &mut Box<dyn super::StatefulNotifier>,
    settings: &settings::Settings,
) -> super::NotificationResult {
    if !n.state().next_reminder_is_due(&ctx.now) {
        // Not yet time to send the next reminder
        return super::NotificationResult::Skipped;
    }

    // Time to send a new reminder, so discard anything old
    n.state_mut().reset();

    match n.push_reminder(ctx) {
        super::NotificationResult::DryRun(message) => {
            println!(
                "[{}] [{}] DRY RUN; not sent",
                utils::timestamp_now(),
                n.name()
            );

            verbose_print(&message, settings);
            n.state_mut().last_reminder_sent = Some(ctx.now);
            n.state_mut().num_consecutive_reminders += 1;
            super::NotificationResult::DryRun(message)
        }
        super::NotificationResult::Success(message) => {
            println!(
                "[{}] [{}] Reminder sent successfully",
                utils::timestamp_now(),
                n.name()
            );

            verbose_print(&message, settings);
            n.state_mut().last_reminder_sent = Some(ctx.now);
            n.state_mut().num_consecutive_reminders += 1;
            super::NotificationResult::Success(message)
        }
        super::NotificationResult::Failure(e, message) => {
            eprintln!(
                "[{}] [{}] Failed to send reminder: {e}",
                utils::timestamp_now(),
                n.name()
            );

            verbose_print(&message, settings);
            n.state_mut().store_notification(ctx, None);
            n.state_mut().num_consecutive_failures += 1;
            super::NotificationResult::Failure(e, message)
        }
        super::NotificationResult::Skipped => {
            // push_reminder does not return Skipped, so this can never happen.
            super::NotificationResult::Skipped
        }
    }
}
