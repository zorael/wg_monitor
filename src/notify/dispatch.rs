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
    let now = time::SystemTime::now();

    let mut report = super::DispatchReport {
        total: notifiers.len() as u32,
        ..Default::default()
    };

    for n in notifiers.iter_mut() {
        if !n
            .state()
            .next_retry_is_due(&now, &settings.monitor.retry_interval)
        {
            // Not yet time
            report.skipped += 1;
            continue;
        }

        // Taking sets pending to None
        let pending = n.state_mut().pending.take();

        let (ctx, delta) = match &pending {
            Some(super::PendingNotification::Notification { context, delta }) => {
                (context, Some(delta))
            }
            Some(super::PendingNotification::Reminder { context }) => (context, None),
            None => {
                // None was taken
                report.skipped += 1;
                continue;
            }
        };

        match send_via_notifier(ctx, delta, n) {
            super::NotificationResult::DryRun(message) => {
                println!(
                    "[{}] [{}] DRY RUN; RETRY not sent",
                    utils::timestamp_now(),
                    n.name()
                );

                verbose_print(&message, settings);
                report.successful += 1;
            }
            super::NotificationResult::Success(message) => {
                println!(
                    "[{}] [{}] Notification RETRIED successfully",
                    utils::timestamp_now(),
                    n.name()
                );

                verbose_print(&message, settings);
                report.successful += 1;
            }
            super::NotificationResult::Failure(e, message) => {
                eprintln!(
                    "[{}] [{}] Failed to RETRY notification: {e}",
                    utils::timestamp_now(),
                    n.name()
                );

                verbose_print(&message, settings);
                report.failed += 1;
            }
            super::NotificationResult::Skipped => {
                // May be due to next [something] not being due yet,
                // so put back the pending notification
                n.state_mut().pending = pending;
                report.skipped += 1;
            }
        }
    }

    if report.total != report.skipped {
        // Linebreak for readability
        println!();
    }

    report
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
        match send_via_notifier(ctx, Some(delta), n) {
            super::NotificationResult::DryRun(message) => {
                println!(
                    "[{}] [{}] DRY RUN; notification not sent",
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

    if report.total != report.skipped {
        // Linebreak for readability
        println!();
    }

    report
}

/// Sends a reminder via all notifiers.
pub fn send_reminder(
    ctx: &super::Context,
    notifiers: &mut [Box<dyn super::StatefulNotifier>],
    settings: &settings::Settings,
) -> super::DispatchReport {
    let mut report = super::DispatchReport {
        total: notifiers.len() as u32,
        ..Default::default()
    };

    for n in notifiers.iter_mut() {
        if !n
            .state()
            .next_reminder_is_due(&ctx.now, &settings.monitor.reminder_interval)
        {
            // Not yet time to send the next reminder
            report.skipped += 1;
            continue;
        }

        match send_via_notifier(ctx, None, n) {
            super::NotificationResult::DryRun(message) => {
                println!(
                    "[{}] [{}] DRY RUN; reminder not sent",
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

    if report.total != report.skipped {
        // Linebreak for readability
        println!();
    }

    report
}

/// Sends either a notification or a reminder via one notifier, depending on
/// whether a delta is provided.
fn send_via_notifier(
    ctx: &super::Context,
    delta: Option<&super::Delta>,
    n: &mut Box<dyn super::StatefulNotifier>,
) -> super::NotificationResult {
    match delta {
        Some(d) => {
            let result = n.push_notification(ctx, d);

            match &result {
                super::NotificationResult::DryRun(_) => {
                    n.state_mut().on_successful_notification(&ctx.now);
                }
                super::NotificationResult::Success(_) => {
                    n.state_mut().on_successful_notification(&ctx.now);
                }
                super::NotificationResult::Failure(_, _) => {
                    n.state_mut().on_failure(ctx, delta, &ctx.now);
                }
                super::NotificationResult::Skipped => {}
            }

            result
        }
        None => {
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
    }
}
