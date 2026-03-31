//! Module responsible for dispatching notifications and reminders via notifiers.

use std::thread;
use std::time;

use crate::defaults;
use crate::logging;
use crate::settings;

/// Helper function to print verbose messages with separators if verbose
/// mode is enabled.
///
/// # Parameters
/// - `message`: The message to print if verbose mode is enabled.
/// - `verbose`: A boolean indicating whether verbose mode is enabled.
fn verbose_print(message: &str, verbose: bool) {
    const SEP: &str = "--------------------";

    if verbose && !message.is_empty() {
        println!("{SEP}\n{}\n{SEP}", message);
    }
}

/// Retries failed notifications that are due for another attempt.
///
/// This function iterates through the provided notifiers, checks if their pending
/// notifications are due for retrying based on the current time and the retry
/// interval specified in settings, and attempts to resend the notifications if they
/// are indeed due.
///
/// # Parameters
/// - `ctx`: The notification context containing information about the current
///   state of peers and other relevant data needed for composing the
///   notification message to retry sending.
/// - `notifiers`: A mutable slice of boxed `StatefulNotifier` instances to check
///   for pending notifications and attempt retries on.
/// - `settings`: The settings struct which contains the retry interval.
///
/// # Returns
/// A `DispatchReport` struct containing the results of the retry attempts,
/// including the total number of notifiers processed, how many were successful,
/// failed, had no message to send, or were skipped due to timing reasons.
pub fn retry_failed_notifications(
    ctx: &super::Context,
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
            // Not yet time to retry
            report.skipped += 1;
            continue;
        }

        // Taking sets it to None, so remember to put it back
        // Make it mutable so we can update the time and iteration for the retry attempt
        let Some(mut failed_ctx) = n.state_mut().failed_ctx.take() else {
            // No failing Context to retry, so skip
            report.skipped += 1;
            continue;
        };

        // Note that this is an Option<KeyDelta>
        let failed_delta = n.state_mut().failed_delta.take();

        // Update the failed Context to the present time and iteration for the retry attempt
        failed_ctx.update_time_and_iteration(now, ctx.loop_iteration);

        match send_via_notifier(&failed_ctx, failed_delta.as_ref(), &ctx.now, n) {
            super::NotificationResult::DryRun(message) => {
                println!();
                logging::tsprintln!(
                    &settings.disable_timestamps,
                    "[{}] DRY RUN; RETRY not sent",
                    n.name()
                );
                verbose_print(&message, settings.verbose);
                report.successful += 1;
            }
            super::NotificationResult::Success(message, output) => {
                println!();
                logging::tsprintln!(
                    &settings.disable_timestamps,
                    "[{}] Notification RETRIED successfully",
                    n.name()
                );

                verbose_print(&message, settings.verbose);
                report.successful += 1;

                if let Some(output) = output
                    && !output.is_empty()
                {
                    logging::tsprintln!(
                        &settings.disable_timestamps,
                        "[{}] Backend output:",
                        n.name()
                    );
                    println!("{output}");
                }
            }
            super::NotificationResult::Failure(e, message) => {
                eprintln!();
                logging::tseprintln!(
                    &settings.disable_timestamps,
                    "[{}] Failed to RETRY notification:",
                    n.name()
                );

                eprintln!("{e}");
                verbose_print(&message, settings.verbose);

                // Put them back and update the notifier state
                let s = n.state_mut();
                s.failed_ctx = Some(failed_ctx);
                s.failed_delta = failed_delta;
                s.last_failed_send = Some(now);
                s.num_consecutive_failures += 1;
                report.failed += 1;
            }
            super::NotificationResult::NoMessage => {
                // Backend returned an empty message, so nothing to send
                report.no_message += 1;
            }
            super::NotificationResult::Skipped => {
                // May be due to next [something] not being due yet,
                // so put back the pending notification
                println!();
                logging::tsprintln!(
                    &settings.disable_timestamps,
                    "[{}] Notification SKIPPED",
                    n.name()
                );

                // Put them back
                let s = n.state_mut();
                s.failed_ctx = Some(failed_ctx);
                s.failed_delta = failed_delta;
                report.skipped += 1;
            }
        }
    }

    report
}

/// Sends an alert notification via all notifiers.
///
/// This function iterates through the provided notifiers and attempts to send an
/// alert using each notifier's `push_alert` method.
///
/// The function also handles the logic for updating the state of
/// each notifier based on the result of the send attempt, such as marking
/// successful alerts or handling failures.
///
/// # Parameters
/// - `ctx`: The notification context containing information about the current
///   state of peers and other relevant data needed for rendering the
///   alert message.
/// - `delta`: The changes detected since the last check,
///   used to determine what has changed.
/// - `notifiers`: A mutable slice of boxed `StatefulNotifier` instances to send
///   the alert through.
/// - `settings`: The settings struct which contains configuration needed for
///   determining how to handle the results of the send attempts.
///
/// # Returns
/// A `DispatchReport` struct containing the results of the send attempts,
/// including the total number of notifiers processed, how many were successful,
/// failed, had no message to send, or were skipped due to timing reasons.
pub fn send_alert(
    ctx: &super::Context,
    delta: &super::KeyDelta,
    notifiers: &mut [Box<dyn super::StatefulNotifier>],
    settings: &settings::Settings,
) -> super::DispatchReport {
    let mut report = super::DispatchReport {
        total: notifiers.len() as u32,
        ..Default::default()
    };

    for n in notifiers.iter_mut() {
        match send_via_notifier(ctx, Some(delta), &ctx.now, n) {
            super::NotificationResult::DryRun(message) => {
                println!();
                logging::tsprintln!(
                    &settings.disable_timestamps,
                    "[{}] DRY RUN; alert not sent",
                    n.name()
                );

                verbose_print(&message, settings.verbose);
                report.successful += 1;
            }
            super::NotificationResult::Success(message, output) => {
                println!();
                logging::tsprintln!(
                    &settings.disable_timestamps,
                    "[{}] Alert sent successfully",
                    n.name()
                );

                verbose_print(&message, settings.verbose);
                report.successful += 1;

                if let Some(output) = output
                    && !output.is_empty()
                {
                    logging::tsprintln!(
                        &settings.disable_timestamps,
                        "[{}] Backend output:",
                        n.name()
                    );
                    println!("{output}");
                }
            }
            super::NotificationResult::Failure(e, message) => {
                eprintln!();
                logging::tseprintln!(
                    &settings.disable_timestamps,
                    "[{}] Failed to send alert:",
                    n.name()
                );
                eprintln!("{e}");

                verbose_print(&message, settings.verbose);
                report.failed += 1;
            }
            super::NotificationResult::NoMessage => {
                // Backend returned an empty message, so nothing to send
                report.no_message += 1;
            }
            super::NotificationResult::Skipped => {
                report.skipped += 1;
            }
        }
    }

    report
}

/// Sends reminders via all notifiers that are due for sending a reminder.
///
/// This function iterates through the provided notifiers, checks if they are due
/// for sending a reminder based on the current time and the reminder interval
/// specified in settings, and attempts to send a reminder if they are due.
///
/// The function also handles the logic for updating the state
/// of each notifier based on the result of the send attempt, such as marking
/// successful reminders or handling failures.
///
/// # Parameters
/// - `ctx`: The notification context containing information about the current
///   state of peers and other relevant data needed for rendering the
///   reminder message.
/// - `notifiers`: A mutable slice of boxed `StatefulNotifier` instances to send
///   the reminder through.
/// - `settings`: The settings struct which contains configuration needed for
///   logging and determining how to handle the results of the send attempts.
///
/// # Returns
/// A `DispatchReport` struct containing the results of the send attempts,
/// including the total number of notifiers processed, how many were successful,
/// failed, had no message to send, or were skipped due to timing.
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

        match send_via_notifier(ctx, None, &ctx.now, n) {
            super::NotificationResult::DryRun(message) => {
                println!();
                logging::tsprintln!(
                    &settings.disable_timestamps,
                    "[{}] DRY RUN; reminder not sent",
                    n.name()
                );

                verbose_print(&message, settings.verbose);
                report.successful += 1;
            }
            super::NotificationResult::Success(message, output) => {
                println!();
                logging::tsprintln!(
                    &settings.disable_timestamps,
                    "[{}] Reminder sent successfully",
                    n.name()
                );

                verbose_print(&message, settings.verbose);
                report.successful += 1;

                if let Some(output) = output
                    && !output.is_empty()
                {
                    logging::tsprintln!(
                        &settings.disable_timestamps,
                        "[{}] Backend output:",
                        n.name()
                    );
                    println!("{output}");
                }
            }
            super::NotificationResult::Failure(e, message) => {
                eprintln!();
                logging::tseprintln!(
                    &settings.disable_timestamps,
                    "[{}] Failed to send reminder:",
                    n.name()
                );
                eprintln!("{e}");

                verbose_print(&message, settings.verbose);
                report.failed += 1;
            }
            super::NotificationResult::NoMessage => {
                // Backend returned an empty message, so nothing to send
                report.no_message += 1;
            }
            super::NotificationResult::Skipped => {
                report.skipped += 1;
            }
        }
    }

    report
}

/// Helper function to send an alert or reminder via a single notifier,
/// and update the notifier's state based on the result.
///
/// # Parameters
/// - `ctx`: The notification context containing information about the current
///   state of peers and other relevant data needed for rendering the
///   alert or reminder message.
/// - `delta`: The changes detected since the last check, used to determine
///   what has changed and render the message accordingly.
///   This will be `None` if sending a reminder instead of an alert.
/// - `now`: The current time, used for updating the notifier's state if the send
///   attempt is successful.
/// - `n`: The notifier to send the alert or reminder through.
///
/// # Returns
/// The result of the send attempt, which can indicate success, failure,
/// a dry run, no message to send, or that the send was skipped due to timing.
fn send_via_notifier(
    ctx: &super::Context,
    delta: Option<&super::KeyDelta>,
    now: &time::SystemTime,
    n: &mut Box<dyn super::StatefulNotifier>,
) -> super::NotificationResult {
    if n.id() > 0 {
        // If this is the second or later notifier of a given backend type,
        // insert a small delay to rate-limit the attempts.
        thread::sleep(defaults::timing::RATE_LIMIT_DELAY_BETWEEN_NOTIFIERS);
    }

    let result = match delta {
        Some(d) => n.push_alert(ctx, d),
        None => n.push_reminder(ctx),
    };

    match &result {
        super::NotificationResult::DryRun(_)
        | super::NotificationResult::Success(_, _)
        | super::NotificationResult::NoMessage => {
            if ctx.has_failed {
                n.state_mut().on_successful_retry();
            } else if delta.is_some() {
                n.state_mut().on_successful_alert(now);
            } else {
                n.state_mut().on_successful_reminder(now);
            }
        }
        super::NotificationResult::Failure(_, _) => {
            if !ctx.has_failed {
                n.state_mut().on_failure(ctx, delta);
            }
        }
        super::NotificationResult::Skipped => {}
    }

    result
}
