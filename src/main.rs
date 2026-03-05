//! Monitors other peers in a [Wireguard VPN](https://www.wireguard.com)
//! and sends a notification if contact with a peer is lost.
//!
//! The main purpose of this is to monitor Internet-connected locations for
//! power outages, using Wireguard handshakes as a way for sites to phone home.
//! Each needs an always-on, always-connected computer to act as a Wireguard
//! peer, for which something like a
//! [Raspberry Pi Zero 2W](https://www.raspberrypi.com/products/raspberry-pi-zero-2-w)
//! is cheap and more than sufficient.
//!
//! In a hub-and-spoke Wireguard configuration, this should be run on the hub
//! server, ideally with an additional instance on (at least) one other
//! geographically disconnected peer to monitor the hub. In other configurations,
//! it can be run on any peer with visibility of other peers, but a secondary
//! instance monitoring the first is recommended in any setup.
//!
//! Peers must have a `PersistentKeepalive` setting in their Wireguard
//! configuration with a value *comfortably lower* than the peer timeout of
//! this program. This timeout is **600 seconds** by default, but can be
//! overridden by modifying a configuration file.
//!
//! Notifications are sent as
//! [**Slack** notifications](https://docs.slack.dev/messaging/sending-messages-using-incoming-webhooks)
//! and/or as short emails via [**Batsign**](https://batsign.me).

mod backend;
mod cli;
mod defaults;
mod file_config;
mod notify;
mod peer;
mod settings;
mod utils;
mod wireguard;

use clap::Parser;
use reqwest::blocking;
use std::fs;
use std::process;
use std::sync::Arc;
use std::thread;
use std::time;

use crate::settings::Settings;

/// Prints a small banner with program metadata.
fn print_banner() {
    println!(
        "{} {} | copyright 2026 {}\n$ git clone {}",
        defaults::program_metadata::PROGRAM_NAME,
        defaults::program_metadata::VERSION,
        defaults::program_metadata::AUTHORS,
        defaults::program_metadata::SOURCE_REPOSITORY
    );
}

/// Program entrypoint.
fn main() -> process::ExitCode {
    print_banner();
    println!();

    let cli = cli::Cli::parse();

    if cli.version {
        // This is the only way to get a neat --version output.
        // The banner with version is already printed just prior to this before clap parses arguments,
        // so we can just exit successfully here after echoing the licenses.
        println!(
            "This project is dual-licensed under the MIT License and \
            the Apache License (Version 2.0) at your option."
        );
        return process::ExitCode::SUCCESS;
    }

    let settings = match init_settings(&cli) {
        Ok(s) => s,
        Err(code) => return code,
    };

    if cli.show {
        // If --show was passed, print settings here and exit early.
        settings.print();
        return process::ExitCode::SUCCESS;
    }

    if let Err(sanity_check_failures) = settings.sanity_check() {
        eprintln!("[X] Configuration has errors:");

        for error in sanity_check_failures {
            eprintln!("  * {error}");
        }

        if settings.dry_run {
            println!("[!] Continuing anyway because --dry-run is set.");
            println!();
        } else {
            return process::ExitCode::from(defaults::exit_codes::CONFIGURATION_ERROR);
        }
    }

    let peers = match wireguard::read_peer_list(&settings.paths.peer_list, settings.debug) {
        Ok(peers) => peers,
        Err(e) => {
            eprintln!("[X] Error reading peers file: {}", e);
            return process::ExitCode::from(defaults::exit_codes::ERROR_READING_PEERS_FILE);
        }
    };

    if peers.is_empty() {
        eprintln!("[X] Peer list file {} is empty.", settings.paths.peer_list.display());
        return process::ExitCode::from(defaults::exit_codes::EMPTY_PEER_LIST);
    }

    // Print resolved settings as part of program startup.
    settings.print();
    println!();

    // Verify that we can execute the `wg show` command but don't actually case
    // about the handshakes at this point. We just want to verify that the
    // command executes successfully before entering the main loop.
    // Exit now if it doesn't.
    let latest_handshakes_output = match wireguard::get_handshakes(&settings.monitor.interface) {
        Ok(output) => {
            /*if settings.debug {
                println!("{output}");
            }*/
            output
        }
        Err(e) => {
            eprintln!("[X] Error executing command: {e}");
            return process::ExitCode::from(
                defaults::exit_codes::FAILED_TO_EXECUTE_HANDSHAKES_COMMAND,
            );
        }
    };

    let handshake_validation_errors = wireguard::validate_handshakes(&latest_handshakes_output);

    if !handshake_validation_errors.is_empty() {
        eprintln!("[X] Error validating latest-handshakes output:");

        for error in handshake_validation_errors {
            eprintln!("  * {error}");
        }

        if settings.dry_run {
            println!("[!] Continuing anyway because --dry-run is set.");
            println!();
        } else {
            return process::ExitCode::from(
                defaults::exit_codes::FAILED_TO_PARSE_HANDSHAKES_OUTPUT,
            );
        }
    }

    let mut notifiers = build_notifiers(&settings);

    if notifiers.is_empty() {
        eprintln!("[X] No notifiers configured.");

        if settings.dry_run {
            println!("[!] Continuing anyway because --dry-run is set.");
            println!();
        } else {
            return process::ExitCode::from(defaults::exit_codes::NO_NOTIFIERS_CONFIGURED);
        }
    }

    // All done, create the initial context and enter the loop.
    let mut ctx = notify::Context::inherit(peers);
    run_loop(&mut ctx, &mut notifiers, settings)
}

/// Construct notifiers based on the passed settings, returning a vector of
/// boxed trait objects.
fn build_notifiers(settings: &Settings) -> Vec<Box<dyn notify::NotificationSender>> {
    let mut notifiers: Vec<Box<dyn notify::NotificationSender>> = Vec::new();
    let client = Arc::new(blocking::Client::new());

    /// Helper function to build and push notifiers for a passed backend type.
    fn build_and_push_notifiers<B, F>(
        vec: &mut Vec<Box<dyn notify::NotificationSender>>,
        urls: &[String],
        mut make_backend_fn: F,
        dry_run: bool,
    ) where
        B: backend::Backend + 'static,
        F: FnMut(usize, &String) -> B, // not &str due to lifetime issues
    {
        for (i, url) in urls.iter().enumerate() {
            let backend = make_backend_fn(i, url);
            let boxed = Box::new(notify::Notifier::new(backend, dry_run));
            vec.push(boxed);
        }
    }

    // Helper closure to build a Slack backend instance.
    let make_slack_backend = |i: usize, url: &String| {
        backend::SlackBackend::new(
            i,
            Arc::clone(&client),
            url,
            &settings.slack.strings,
            &settings.slack.reminder_strings,
        )
    };

    // Helper closure to build a Batsign backend instance.
    let make_batsign_backend = |i: usize, url: &String| {
        backend::BatsignBackend::new(
            i,
            Arc::clone(&client),
            url,
            &settings.batsign.strings,
            &settings.batsign.reminder_strings,
        )
    };

    if settings.dry_run {
        // Use dummy URLs for dry runs so that we can get output for all backends
        // even if no URLs were configured.
        let dummy_slack_urls = vec![defaults::DUMMY_SLACK_URL.to_string()];
        let dummy_batsign_urls = vec![defaults::DUMMY_BATSIGN_URL.to_string()];

        build_and_push_notifiers(&mut notifiers, &dummy_slack_urls, make_slack_backend, true);
        build_and_push_notifiers(
            &mut notifiers,
            &dummy_batsign_urls,
            make_batsign_backend,
            true,
        );
    } else {
        if settings.slack.enabled && !settings.slack.urls.is_empty() {
            build_and_push_notifiers(
                &mut notifiers,
                &settings.slack.urls,
                make_slack_backend,
                false,
            );
        }

        if settings.batsign.enabled && !settings.batsign.urls.is_empty() {
            build_and_push_notifiers(
                &mut notifiers,
                &settings.batsign.urls,
                make_batsign_backend,
                false,
            )
        }
    }

    notifiers
}

fn retry_stored_notifications(
    notifiers: &mut [Box<dyn notify::NotificationSender>],
    settings: &Settings,
) -> notify::DispatchReport {
    let mut report = notify::DispatchReport {
        total: notifiers.len() as u32,
        ..Default::default()
    };

    fn verbose_print(message: &Option<String>, settings: &Settings) {
        const SEP: &str = "--------------------";

        if settings.verbose
            && let Some(msg) = message
        {
            println!("{SEP}\n{msg}\n{SEP}");
        }
    }

    for n in notifiers.iter_mut() {
        let (result, message) = retry_single_notification(n, settings);

        match result {
            notify::NotificationResult::DryRun => {
                verbose_print(&message, settings);
                report.successful += 1;
            }
            notify::NotificationResult::Success => {
                verbose_print(&message, settings);
                report.successful += 1;
            }
            notify::NotificationResult::Failure(_) => {
                verbose_print(&message, settings);
                report.failed += 1;
            }
            notify::NotificationResult::Skipped => {
                verbose_print(&message, settings);
                report.skipped += 1;
            }
        }
    }

    report
}

fn retry_single_notification(
    n: &mut Box<dyn notify::NotificationSender>,
    settings: &Settings,
) -> (notify::NotificationResult, Option<String>) {
    fn verbose_print(message: &str, settings: &Settings) {
        if settings.verbose {
            const SEP: &str = "--------------------";
            println!("{SEP}\n{message}\n{SEP}");
        }
    }

    match n.get_stored_notification() {
        // If it has a Context and a Delta, it is a notification
        // If it only has a Context, it is a reminder
        // If it has neither, it doesn't have a stored notification
        (Some(ctx), Some(delta)) => {
            match n.push_notification(&ctx, &delta) {
                (notify::NotificationResult::DryRun, message) => {
                    println!("[{}] DRY RUN", n.name());
                    verbose_print(&message, settings);
                    n.clear_stored_notification(); // Notification sent so discard it
                    (notify::NotificationResult::DryRun, Some(message))
                }
                (notify::NotificationResult::Success, message) => {
                    println!("[{}] Notification sent successfully", n.name());
                    verbose_print(&message, settings);
                    n.clear_stored_notification(); // As above, discard it
                    (notify::NotificationResult::Success, Some(message))
                }
                (notify::NotificationResult::Failure(e), message) => {
                    eprintln!("[{}] Failed to send notification: {e}", n.name());
                    verbose_print(&message, settings);
                    (notify::NotificationResult::Failure(e), Some(message))
                }
                _ => {
                    // Should never happen.
                    (notify::NotificationResult::Skipped, None)
                }
            }
        }
        (Some(ctx), None) => match n.push_reminder(&ctx) {
            (notify::NotificationResult::DryRun, message) => {
                println!("[{}] DRY RUN", n.name());
                verbose_print(&message, settings);
                n.clear_stored_notification(); // Reminder sent so discard it
                n.increment_num_consecutive_reminders();
                (notify::NotificationResult::DryRun, Some(message))
            }
            (notify::NotificationResult::Success, message) => {
                println!("[{}] Reminder sent successfully", n.name());
                verbose_print(&message, settings);
                n.clear_stored_notification(); // As above
                n.set_last_reminder_sent(Some(ctx.now));
                n.increment_num_consecutive_reminders();
                (notify::NotificationResult::Success, Some(message))
            }
            (notify::NotificationResult::Failure(e), message) => {
                eprintln!("[{}] Failed to send reminder: {e}", n.name());
                verbose_print(&message, settings);
                (notify::NotificationResult::Failure(e), Some(message))
            }
            _ => {
                // Should never happen.
                (notify::NotificationResult::Skipped, None)
            }
        },
        (None, _) => (notify::NotificationResult::Skipped, None),
    }
}

fn send_notification(
    ctx: &notify::Context,
    delta: &notify::Delta,
    notifiers: &mut [Box<dyn notify::NotificationSender>],
    settings: &Settings,
) -> notify::DispatchReport {
    let mut report = notify::DispatchReport {
        total: notifiers.len() as u32,
        ..Default::default()
    };

    for n in notifiers.iter_mut() {
        let (result, _) = send_single_notifier_notification(ctx, delta, n, settings);

        match result {
            notify::NotificationResult::DryRun => {
                report.successful += 1;
            }
            notify::NotificationResult::Success => {
                report.successful += 1;
            }
            notify::NotificationResult::Failure(_) => {
                report.failed += 1;
            }
            notify::NotificationResult::Skipped => {
                report.skipped += 1;
            }
        }
    }

    report
}

fn send_single_notifier_notification(
    ctx: &notify::Context,
    delta: &notify::Delta,
    n: &mut Box<dyn notify::NotificationSender>,
    settings: &Settings,
) -> (notify::NotificationResult, Option<String>) {
    fn verbose_print(message: &str, settings: &Settings) {
        if settings.verbose {
            const SEP: &str = "--------------------";
            println!("{SEP}\n{message}\n{SEP}");
        }
    }

    // Time to send a new notification, so discard any stored ones
    n.clear_stored_notification();
    n.reset_num_consecutive_reminders();
    n.clear_last_reminder_sent();

    match n.push_notification(ctx, delta) {
        (notify::NotificationResult::DryRun, message) => {
            println!("[{}] DRY RUN", n.name());
            verbose_print(&message, settings);
            (notify::NotificationResult::DryRun, Some(message))
        }
        (notify::NotificationResult::Success, message) => {
            println!("[{}] Notification sent successfully", n.name());
            verbose_print(&message, settings);
            (notify::NotificationResult::Success, Some(message))
        }
        (notify::NotificationResult::Failure(e), message) => {
            eprintln!("[{}] Failed to send notification: {e}", n.name());
            verbose_print(&message, settings);
            n.store_notification(ctx, Some(delta)); // Store the failure for retrying
            (notify::NotificationResult::Failure(e), Some(message))
        }
        _ => {
            // Should never happen.
            (notify::NotificationResult::Skipped, None)
        }
    }
}

fn send_single_notifier_reminder(
    ctx: &notify::Context,
    n: &mut Box<dyn notify::NotificationSender>,
    settings: &Settings,
) -> (notify::NotificationResult, Option<String>) {
    fn verbose_print(message: &str, settings: &Settings) {
        if settings.verbose {
            const SEP: &str = "--------------------";
            println!("{SEP}\n{message}\n{SEP}");
        }
    }

    // Time to send a new reminder, so discard any stored ones
    n.clear_stored_notification();

    match n.push_reminder(ctx) {
        (notify::NotificationResult::DryRun, message) => {
            println!("[{}] DRY RUN", n.name());
            verbose_print(&message, settings);
            n.set_last_reminder_sent(Some(ctx.now));
            n.increment_num_consecutive_reminders();
            (notify::NotificationResult::DryRun, Some(message))
        }
        (notify::NotificationResult::Success, message) => {
            println!("[{}] Reminder sent successfully", n.name());
            verbose_print(&message, settings);
            n.set_last_reminder_sent(Some(ctx.now));
            n.increment_num_consecutive_reminders();
            (notify::NotificationResult::Success, Some(message))
        }
        (notify::NotificationResult::Failure(e), message) => {
            eprintln!("[{}] Failed to send reminder: {e}", n.name());
            verbose_print(&message, settings);
            n.store_notification(ctx, None);
            (notify::NotificationResult::Failure(e), Some(message))
        }
        _ => {
            // Should never happen.
            (notify::NotificationResult::Skipped, None)
        }
    }
}

/// Main loop of the program.
fn run_loop(
    ctx: &mut notify::Context,
    notifiers: &mut [Box<dyn notify::NotificationSender>],
    settings: Settings,
) -> process::ExitCode {
    /// Perform some cleanup and sleep at the end of each loop duration.
    fn end_loop(ctx: &mut notify::Context, duration: time::Duration) {
        ctx.rotate();
        ctx.first_run = false;
        ctx.resume = false;
        thread::sleep(duration);
    }

    let mut delta = notify::Delta::with_capacity(ctx.peers.len());
    let mut should_skip_next = settings.skip_first;

    // If `resume` is set, we want to skip the first run. The easiest way is to
    // just set `first_run` to `false` here.
    ctx.first_run = !settings.resume;
    ctx.resume = settings.resume;

    loop {
        match wireguard::get_handshakes(&settings.monitor.interface) {
            Ok(output) => {
                if settings.debug {
                    // This is very spammy so gate it behind debug instead of verbose mode.
                    println!("{output}");
                }
                wireguard::update_handshakes(&output, &mut ctx.peers);
            }
            Err(e) => {
                eprintln!("[!] Error executing command: {e}");
                thread::sleep(settings.monitor.check_interval);
                continue;
            }
        };

        ctx.now = time::SystemTime::now();

        for (key, peer) in ctx.peers.iter() {
            match peer.last_seen {
                Some(last_seen) => {
                    let age = ctx
                        .now
                        .duration_since(last_seen)
                        .unwrap_or(time::Duration::ZERO);

                    if settings.debug {
                        let dt: chrono::DateTime<chrono::Local> = last_seen.into();
                        println!(
                            "  * Peer '{}': last seen {} seconds ago at {}",
                            peer.human_name,
                            age.as_secs(),
                            dt.format("%Y-%m-%d %H:%M:%S")
                        );
                    }

                    if age < settings.monitor.timeout {
                        continue;
                    }

                    if settings.debug {
                        println!("... age is greater than timeout, marking as late");
                    }

                    ctx.late_keys.push(key.clone());
                }
                None => {
                    if settings.debug {
                        println!(
                            "  * Peer '{}' has never been seen, marking as missing",
                            peer.human_name
                        );
                    }

                    ctx.missing_keys.push(key.clone());
                }
            }
        }

        delta.compute_from(ctx);

        // --skip-first logic is here
        if should_skip_next {
            should_skip_next = false;
            end_loop(ctx, settings.monitor.check_interval);
            continue;
        }

        if delta.is_empty() {
            if ctx.missing_keys.is_empty() && ctx.late_keys.is_empty() {
                // End of the line but there may be stored notifications
                let report = retry_stored_notifications(notifiers, &settings);

                if settings.debug && report.total != report.skipped {
                    println!("{:#?}", report);
                }

                end_loop(ctx, settings.monitor.check_interval);
                continue;
            }

            // No changes but there are missing/late peers, so we may need to send reminders
            for n in notifiers.iter_mut() {
                if let Some(last_reminder_sent) = n.get_last_reminder_sent() {
                    // Grow the reminder interval over time but cap it at 48h
                    let growth_multiplier = match n.get_num_consecutive_reminders() {
                        0 => 1, // 6h (assuming default reminder interval)
                        1 => 2, // 12h
                        2 => 2, // 12h
                        3 => 4, // 24h
                        4 => 4, // 24h
                        _ => 8, // 48h
                    };

                    let next_report_interval =
                        growth_multiplier * settings.monitor.reminder_interval;

                    if ctx
                        .now
                        .duration_since(last_reminder_sent)
                        .unwrap_or(time::Duration::ZERO)
                        > next_report_interval
                    {
                        let (result, _) = send_single_notifier_reminder(ctx, n, &settings);

                        if settings.debug {
                            println!("{:#?}", result);
                        }
                    }
                }
            }

            end_loop(ctx, settings.monitor.check_interval);
            continue;
        }

        if settings.debug {
            delta.print_nonempty_prefixed("... ");
        }

        //let report = notify::send_notification(notifiers, ctx, &delta, settings.verbose);
        let report = send_notification(ctx, &delta, notifiers, &settings);

        if settings.debug && report.total != report.skipped {
            println!("{:#?}\n", report);
        }

        end_loop(ctx, settings.monitor.check_interval);
    }
}

/// Initializes all settings, except for CLI parsing, which must already have been done.
fn init_settings(cli: &cli::Cli) -> Result<Settings, process::ExitCode> {
    let mut settings = Settings::default();

    if let Err(e) = settings.inherit_config_dir(&cli.config_dir) {
        eprintln!("[X] Error resolving default configuration directory: {}", e);
        return Err(process::ExitCode::from(
            defaults::exit_codes::FAILED_TO_RESOLVE_CONFIG_DIR,
        ));
    }

    if !settings.paths.config_dir.exists() && !cli.save {
        eprintln!(
            "[X] Configuration directory {} does not exist. \
            Create it or run with `--save` to generate default configuration and resources.",
            settings.paths.config_dir.display()
        );
        return Err(process::ExitCode::from(
            defaults::exit_codes::CONFIG_DIR_DOES_NOT_EXIST,
        ));
    }

    settings.resolve_resource_paths();

    let config = match file_config::deserialize_config_file(&settings.paths.config_file) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!(
                "[X] Failed to read configuration file {}: {e}",
                settings.paths.config_file.display()
            );
            return Err(process::ExitCode::from(
                defaults::exit_codes::FAILED_TO_READ_CONFIG_FILE,
            ));
        }
    };

    if !cli.save && config.is_none() {
        eprintln!(
            "[X] No configuration file found at {}. \
            Create it or run with `--save` to generate default configuration and resources.",
            settings.paths.config_file.display()
        );
        return Err(process::ExitCode::from(
            defaults::exit_codes::CONFIG_FILE_DOES_NOT_EXIST,
        ));
    }

    settings.apply_file(&config);
    settings.apply_cli(cli);
    settings.clean_up();

    if cli.save {
        if !settings.paths.config_dir.exists() {
            match fs::create_dir_all(&settings.paths.config_dir) {
                Ok(()) => {
                    println!(
                        "[O] Configuration directory {} created.",
                        settings.paths.config_dir.display()
                    );
                }
                Err(e) => {
                    eprintln!(
                        "[X] Failed to create configuration directory {}: {e}",
                        settings.paths.config_dir.display()
                    );

                    return Err(process::ExitCode::from(
                        defaults::exit_codes::FAILED_TO_CREATE_CONFIG_DIR,
                    ));
                }
            };
        }

        let config = file_config::FileConfig::from(&settings);

        if let Err(e) = confy::store_path(&settings.paths.config_file, config) {
            eprintln!(
                "[X] Failed to write configuration file {}: {e}",
                settings.paths.config_file.display()
            );

            return Err(process::ExitCode::from(
                defaults::exit_codes::FAILED_TO_WRITE_CONFIG_FILE,
            ));
        };

        if !settings.paths.peer_list.exists() {
            match fs::write(&settings.paths.peer_list, defaults::EMPTY_PEER_LIST_CONTENT) {
                Ok(()) => {
                    println!(
                        "[O] Empty peer list file {} created.",
                        settings.paths.peer_list.display()
                    );
                }
                Err(e) => {
                    eprintln!(
                        "[X] Failed to write empty peer list file {}: {e}",
                        &settings.paths.peer_list.display()
                    );

                    return Err(process::ExitCode::from(
                        defaults::exit_codes::FAILED_TO_WRITE_PEER_LIST_FILE,
                    ));
                }
            };
        }

        println!(
            "[O] Configuration and resources written successfully to {}.",
            settings.paths.config_dir.display()
        );
        return Err(process::ExitCode::SUCCESS);
    }

    Ok(settings)
}
