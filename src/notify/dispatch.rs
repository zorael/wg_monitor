//! Module responsible for dispatching notifications and reminders via notifiers.

use std::time;

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

/// Retries pending notifications that are due for retrying, and updates the
/// report with the results of the retry attempts.
///
/// This function iterates through the provided notifiers, checks if their pending
/// notifications are due for retrying based on the current time and the retry
/// interval specified in settings, and attempts to resend the notifications if they
/// are indeed due.
///
/// # Parameters
/// - `notifiers`: A mutable slice of boxed `StatefulNotifier` instances to check
///   for pending notifications and attempt retries on.
/// - `settings`: The settings struct which contains the retry interval.
///
/// # Returns
/// A `DispatchReport` struct containing the results of the retry attempts,
/// including the total number of notifiers processed, how many were successful,
/// failed, had no message to send, or were skipped due to timing reasons.
pub fn retry_pending_notifications(
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
            // Not yet time to retry the pending notification
            report.skipped += 1;
            continue;
        }

        // Taking sets it to None, so remember to put it back
        let Some(first_failed_ctx) = n.state_mut().first_failed_ctx.take() else {
            // No pending notification to retry
            report.skipped += 1;
            continue;
        };

        let failed_delta = n.state_mut().first_failed_delta.take();

        let modified_failed_ctx = first_failed_ctx.clone();
        let modified_failed_ctx = super::Context {
            peers: modified_failed_ctx.peers,
            lost_keys: modified_failed_ctx.lost_keys,
            missing_keys: modified_failed_ctx.missing_keys,
            now,                                // <-- changed
            loop_iteration: ctx.loop_iteration, // <-- changed
            resume: false,
            peer_list: modified_failed_ctx.peer_list,
        };

        match send_via_notifier(&modified_failed_ctx, failed_delta.as_ref(), &ctx.now, n) {
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

                // Put it back
                n.state_mut().first_failed_ctx = Some(first_failed_ctx);
                n.state_mut().first_failed_delta = failed_delta;
                verbose_print(&message, settings.verbose);
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

                // Put it back
                n.state_mut().first_failed_ctx = Some(first_failed_ctx);
                n.state_mut().first_failed_delta = failed_delta;
                report.skipped += 1;
            }
        }
    }

    report
}

#[cfg(false)]
pub fn retry_pending_notifications2(
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
        /*if !n
            .state()
            .next_retry_is_due(&now, &settings.monitor.retry_interval)
        {
            // Not yet time
            report.skipped += 1;
            continue;
        }*/

        // Taking sets it to None, so remember to put it back
        let Some(first_failed_ctx) = n.state_mut().first_failed_ctx.take() else {
            // No pending notification to retry
            report.skipped += 1;
            continue;
        };

        let failed_ctx = first_failed_ctx.clone();
        let orig_delta = super::Context::delta_between(ctx, &failed_ctx);

        let currently_lost_keys =
            utils::get_elements_not_in_other_vec(&failed_ctx.lost_keys, &orig_delta.was_lost);

        let currently_missing_keys =
            utils::get_elements_not_in_other_vec(&failed_ctx.missing_keys, &orig_delta.was_missing);

        let failed_ctx = super::Context {
            peers: failed_ctx.peers,
            lost_keys: failed_ctx.lost_keys, //currently_lost_keys,
            missing_keys: failed_ctx.missing_keys, //currently_missing_keys,
            now,
            loop_iteration: ctx.loop_iteration, // <--
            resume: ctx.resume,
            peer_list: failed_ctx.peer_list,
        };

        /*let delta = match super::Context::delta_between(ctx, &failed_ctx) {
            delta if delta.is_empty() => delta,
            delta => orig_delta,
        };*/

        println!(
            "consecutive notif: {}, reminders: {}",
            n.state().get_consecutive_notifications(),
            n.state().get_consecutive_reminders(),
        );

        let at_least_one_successful_message_has_been_sent =
            n.state().get_consecutive_notifications() > 0;
        let at_least_one_successful_reminder_has_been_sent =
            n.state().get_consecutive_reminders() > 0;
        let the_retry_is_not_of_the_first_message = at_least_one_successful_message_has_been_sent
            || at_least_one_successful_reminder_has_been_sent;

        let delta = match super::Context::delta_between(ctx, &failed_ctx) {
            delta if delta.is_empty() => {
                println!("NEW: {:?}", delta);
                println!("ORIG:{:?}", orig_delta);
                if the_retry_is_not_of_the_first_message {
                    println!("_________ not the first message");
                    // Make it a reminder
                    Some(delta)
                } else {
                    println!("__________________ IS the first message");
                    Some(orig_delta)
                }
            }
            _ => Some(orig_delta),
        };

        // Wrap it in an Option<&KeyDelta> to pass to send_via_notifier
        /*let delta = match delta {
            delta if delta.is_empty() => None,
            delta => Some(&delta.clone()),
        };*/

        match send_via_notifier(&failed_ctx, delta.as_ref(), &ctx.now, n) {
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

                // Put it back
                n.state_mut().first_failed_ctx = Some(first_failed_ctx);
                verbose_print(&message, settings.verbose);
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

                // Put it back
                n.state_mut().first_failed_ctx = Some(first_failed_ctx);
                report.skipped += 1;
            }
        }
    }

    report
}

/// Sends a notification via all notifiers.
///
/// This function iterates through the provided notifiers and attempts to send a
/// notification using each notifier's `push_notification` method.
///
/// The function also handles the logic for updating the state of
/// each notifier based on the result of the send attempt, such as marking
/// successful notifications or handling failures.
///
/// # Parameters
/// - `ctx`: The notification context containing information about the current
///   state of peers and other relevant data needed for rendering the
///   notification message.
/// - `delta`: The changes detected since the last notification,
///   used to determine what has changed.
/// - `notifiers`: A mutable slice of boxed `StatefulNotifier` instances to send
///   the notification through.
/// - `settings`: The settings struct which contains configuration needed for
///   determining how to handle the results of the send attempts.
///
/// # Returns
/// A `DispatchReport` struct containing the results of the send attempts,
/// including the total number of notifiers processed, how many were successful,
/// failed, had no message to send, or were skipped due to timing reasons.
pub fn send_notification(
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
                    "[{}] DRY RUN; notification not sent",
                    n.name()
                );

                verbose_print(&message, settings.verbose);
                report.successful += 1;
            }
            super::NotificationResult::Success(message, output) => {
                println!();
                logging::tsprintln!(
                    &settings.disable_timestamps,
                    "[{}] Notification sent successfully",
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
                    "[{}] Failed to send notification:",
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

/// Helper function to send a notification or reminder via a single notifier,
/// and update the notifier's state based on the result.
///
/// # Parameters
/// - `ctx`: The notification context containing information about the current
///   state of peers and other relevant data needed for rendering the
///   notification or reminder message.
/// - `delta`: The changes detected since the last notification, used to determine
///   what has changed and render the message accordingly.
///   This will be `None` if sending a reminder instead of a notification.
/// - `n`: The notifier to send the notification or reminder through.
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
    let result = match delta {
        Some(d) => n.push_notification(ctx, d),
        None => n.push_reminder(ctx),
    };

    println!("push_notification/reminder result: {:?}", result);

    match &result {
        super::NotificationResult::DryRun(_)
        | super::NotificationResult::Success(_, _)
        | super::NotificationResult::NoMessage => {
            if delta.is_some() {
                n.state_mut().on_successful_notification(now);
            } else {
                n.state_mut().on_successful_reminder(now);
            }
        }
        super::NotificationResult::Failure(_, _) => {
            n.state_mut().on_failure(ctx, delta);
        }
        super::NotificationResult::Skipped => {}
    }

    result
}
