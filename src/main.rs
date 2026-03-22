//! Monitors other peers in a [**WireGuard**](https://www.wireguard.com) VPN
//! and sends a notification if contact with a peer is lost.
//!
//! The main purpose of this is to monitor Internet-connected locations for
//! power outages, using WireGuard handshakes as a way for sites to phone home.
//! Each needs an always-on, always-connected computer to act as a WireGuard
//! peer, for which something like a
//! [Raspberry Pi Zero 2W](https://www.raspberrypi.com/products/raspberry-pi-zero-2-w)
//! is cheap and more than sufficient. (May require [cross-compilation](#cross-compilation).)
//!
//! In a hub-and-spoke WireGuard configuration, this should be run on the hub
//! server, ideally with an additional instance on (at least) one other
//! geographically disconnected peer to monitor the hub. In other configurations,
//! it can be run on any peer with visibility of other peers, but a secondary
//! instance monitoring the first is recommended in any setup.
//!
//! Peers must have a `PersistentKeepalive` setting in their WireGuard
//! configuration with a value *comfortably lower* than the peer timeout of
//! this program. This timeout is **600 seconds** by default, but can be
//! overridden by modifying a configuration file.
//!
//! Notifications can be sent as
//! [**Slack**](https://docs.slack.dev/messaging/sending-messages-using-incoming-webhooks)
//! messages, as short emails via [**Batsign**](https://batsign.me), and/or by
//! invocation of an [**external command**](#external-command)
//! (like `notify-send` or `sendmail`).
//!
//! At any given time, peers can be in one of three states:
//!
//! - **present**: the peer has been seen within the timeout period.
//! - **late**: the peer has been seen before but has not been seen within the
//!   timeout period. It may be referred to as "lost" in some message strings.
//! - **missing**: the peer has not been seen since the last restart of the VPN.
//!
//! As such, peer may be in the following transition states;
//!
//! - **became late**: the peer was present but is now late. This may be referred
//!   to as "lost" in some message strings.
//! - **went missing**: the peer was present but is now missing, which is usually
//!   indicative of a restart of the VPN. This may be referred to as "lost due
//!   to a network reset" in some message strings.
//! - **no longer late**: the peer was late but is now present again. This may
//!   be referred to as "returned" in some message strings.
//! - **returned**: the peer was missing (never seen) but is now present.
//!   This may be referred to as "just appeared" in some message strings.

mod backend;
mod cli;
mod defaults;
mod file_config;
mod logging;
mod notify;
mod peer;
mod settings;
mod utils;
mod wireguard;

use clap::Parser;
use std::fs;
use std::process;
use std::thread;
use std::time;

/// Prints the program banner with name, version, copyright and source repository.
fn print_banner() {
    println!(
        "{} {} | copyright 2026 {}\n$ git clone {}",
        defaults::program_metadata::PROGRAM_NAME,
        defaults::program_metadata::VERSION,
        defaults::program_metadata::AUTHORS,
        defaults::program_metadata::SOURCE_REPOSITORY
    );
}

/// Main entrypoint of the program.
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
        logging::tseprintln!(&settings, "Incomplete or invalid configuration:");

        for error in sanity_check_failures {
            eprintln!("  * {error}");
        }

        if settings.dry_run {
            logging::tsprintln!(
                &settings,
                "Continuing anyway because --dry-run was supplied."
            );
        } else {
            return process::ExitCode::from(defaults::exit_codes::CONFIGURATION_ERROR);
        }
    }

    let peers = match wireguard::read_peer_list(&settings.paths.peer_list, settings.debug) {
        Ok(peers) => peers,
        Err(e) => {
            logging::tseprintln!(&settings, "Error reading peers file: {e}");
            return process::ExitCode::from(defaults::exit_codes::ERROR_READING_PEERS_FILE);
        }
    };

    if peers.is_empty() {
        logging::tseprintln!(
            &settings,
            "Peer list file {} is empty.",
            settings.paths.peer_list.display()
        );
        return process::ExitCode::from(defaults::exit_codes::EMPTY_PEER_LIST);
    }

    // Verify that we can execute the `wg show` command but don't actually care
    // about the handshakes at this point. We just want to verify that the
    // command executes successfully before entering the main loop.
    let latest_handshakes_output = loop {
        match wireguard::get_handshakes(&settings.monitor.interface) {
            Ok(output) => break output,
            Err(e) => {
                let e = e.to_string();
                logging::tseprintln!(&settings, "{e}");

                if e.contains("No such device") {
                    logging::tsprintln!(
                        &settings,
                        "Interface {} down? Retrying in {}...",
                        settings.monitor.interface,
                        humantime::format_duration(settings.monitor.check_interval)
                    );

                    thread::sleep(settings.monitor.check_interval);
                    continue;
                } else if e.contains("Operation not permitted") {
                    logging::tseprintln!(
                        &settings,
                        "Insufficient privileges to execute 'wg show' command."
                    );
                    return process::ExitCode::from(defaults::exit_codes::INSUFFICIENT_PRIVILEGES);
                } else {
                    logging::tseprintln!(&settings, "Failed to execute handshakes command.");
                    return process::ExitCode::from(
                        defaults::exit_codes::FAILED_TO_EXECUTE_HANDSHAKES_COMMAND,
                    );
                }
            }
        };
    };

    let handshake_validation_errors = wireguard::validate_handshakes(&latest_handshakes_output);

    if !handshake_validation_errors.is_empty() {
        logging::tseprintln!(&settings, "Error validating latest-handshakes output:",);

        for error in handshake_validation_errors {
            eprintln!("  * {error}");
        }

        if settings.dry_run {
            logging::tsprintln!(&settings, "Continuing anyway because --dry-run is set.",);
            println!();
        } else {
            return process::ExitCode::from(
                defaults::exit_codes::FAILED_TO_PARSE_HANDSHAKES_OUTPUT,
            );
        }
    }

    let mut notifiers = build_notifiers(&settings);

    if notifiers.is_empty() {
        logging::tseprintln!(&settings, "No notifiers configured.");

        if settings.dry_run {
            logging::tsprintln!(&settings, "Continuing anyway because --dry-run is set.",);
            println!();
        } else {
            return process::ExitCode::from(defaults::exit_codes::NO_NOTIFIERS_CONFIGURED);
        }
    }

    logging::tsprintln!(&settings, "Initialization complete.");

    if settings.debug {
        println!("\n{:#?}\n", settings);
    } else {
        println!();
        println!(
            "{} peer(s) monitored, {} notifier(s) configured.",
            peers.len(),
            notifiers.len()
        );
        println!(
            "check interval: {}, peer timeout: {}",
            humantime::format_duration(settings.monitor.check_interval),
            humantime::format_duration(settings.monitor.timeout),
        );
        println!(
            "reminder interval: {}, retry interval: {}",
            humantime::format_duration(settings.monitor.reminder_interval),
            humantime::format_duration(settings.monitor.retry_interval),
        );
        println!();

        if settings.dry_run {
            logging::tsprintln!(&settings, "DRY RUN");
        }
    }

    // All done, create the initial context
    let mut ctx = notify::Context::inherit(peers);

    // Store the peer list file path in the context so that backends can access it
    ctx.peer_list_file_path = settings.paths.peer_list.display().to_string();

    // And finally enter the loop.
    logging::tsprintln!(&settings, "Entering main loop...");
    run_loop(&mut ctx, &mut notifiers, settings)
}

/// Builds notifiers for all configured backends and returns them as a vector
/// of trait objects.
///
/// This function handles both the normal and dry-run cases, using dummy
/// URLs/commands for the latter to allow testing of notification logic
/// without actual external dependencies.
///
/// # Parameters
/// - `settings`: The program settings, used to determine which backends are
///   enabled and to access necessary configuration for each backend.
///
/// # Returns
/// A vector of boxed `StatefulNotifier` trait objects, each wrapping a notifier
/// for a configured backend.
fn build_notifiers(settings: &settings::Settings) -> Vec<Box<dyn notify::StatefulNotifier>> {
    let mut notifiers: Vec<Box<dyn notify::StatefulNotifier>> = Vec::new();
    let agent = ureq::Agent::new_with_defaults();

    /// Helper function to build and push notifiers for a passed backend type.
    fn build_and_push_notifiers<B, F>(
        vec: &mut Vec<Box<dyn notify::StatefulNotifier>>,
        elements: &[String],
        mut make_backend_fn: F,
        dry_run: bool,
    ) where
        B: backend::Backend + 'static,
        F: FnMut(usize, &String) -> B, // not &str due to lifetime issues
    {
        for (i, element) in elements.iter().enumerate() {
            let backend = make_backend_fn(i, element);
            let boxed = Box::new(notify::Notifier::new(backend, dry_run));
            vec.push(boxed);
        }
    }

    // Helper closure to build a Slack backend instance.
    let make_slack_backend = |i: usize, url: &String| {
        backend::SlackBackend::new(
            i,
            agent.clone(),
            url,
            &settings.slack.strings,
            &settings.slack.reminder_strings,
        )
    };

    // Helper closure to build a Batsign backend instance.
    let make_batsign_backend = |i: usize, url: &String| {
        backend::BatsignBackend::new(
            i,
            agent.clone(),
            url,
            &settings.batsign.strings,
            &settings.batsign.reminder_strings,
        )
    };

    let make_command_backend = |i: usize, command: &String| {
        backend::CommandBackend::new(
            i,
            command,
            &settings.command.strings,
            &settings.command.reminder_strings,
        )
    };

    if settings.dry_run {
        // Use dummy URLs for dry runs so that we can get output for all backends
        // even if no URLs were configured.
        let dummy_slack_urls = vec![defaults::DUMMY_SLACK_URL.to_string()];
        let dummy_batsign_urls = vec![defaults::DUMMY_BATSIGN_URL.to_string()];
        let dummy_command = vec![defaults::DUMMY_COMMAND.to_string()];

        build_and_push_notifiers(&mut notifiers, &dummy_slack_urls, make_slack_backend, true);
        build_and_push_notifiers(
            &mut notifiers,
            &dummy_batsign_urls,
            make_batsign_backend,
            true,
        );
        build_and_push_notifiers(&mut notifiers, &dummy_command, make_command_backend, true);
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

        if settings.command.enabled && !settings.command.commands.is_empty() {
            build_and_push_notifiers(
                &mut notifiers,
                &settings.command.commands,
                make_command_backend,
                false,
            )
        }
    }

    notifiers
}

/// Runs the main monitoring loop, which continuously checks the status of peers
/// and sends notifications as needed.
///
/// This function will run indefinitely until the program is terminated. It performs
/// the following steps in each iteration:
///
/// 1. Executes the `wg show` command to get the latest handshakes and updates
///    the context with the new information.
/// 2. Determines which peers are late or missing based on the last see
///    timestamps and the configured timeout.
/// 3. Computes the delta of changes since the last iteration.
/// 4. If there are changes, sends notifications through the configured notifiers.
/// 5. If there are no changes but there are late/missing peers,
///    sends reminders as needed.
/// 6. Retries any pending notifications that are due for a retry.
/// 7. Sleeps for the configured check interval before the next iteration.
///
/// # Parameters
/// - `ctx`: The notification context, which holds the current state of peers
///   and other relevant information.
/// - `notifiers`: A mutable slice of stateful notifiers to use for
///   sending notifications
/// - `settings`: The program settings, used to determine behavior such as
///   intervals and debug output.
///
/// # Returns
/// This function will never return under normal operation, as it runs an
/// infinite loop. It will only return an exit code if the loop is somehow
/// exited, which would typically indicate a shutdown or critical failure.
/// In normal operation, the program should be terminated externally
/// (via a signal) rather than exiting this function.
fn run_loop(
    ctx: &mut notify::Context,
    notifiers: &mut [Box<dyn notify::StatefulNotifier>],
    settings: settings::Settings,
) -> process::ExitCode {
    /// Perform some cleanup and sleep at the end of each loop duration.
    fn end_loop(ctx: &mut notify::Context, duration: time::Duration) {
        ctx.rotate();
        ctx.resume = false;
        ctx.loop_iteration += 1;
        thread::sleep(duration);
    }

    let mut delta = notify::Delta::with_capacity(ctx.peers.len());
    let mut should_skip_next = settings.skip_first;

    // If `resume` is set, we want to skip the first run. The easiest way is to
    // just set start `loop_iteration` at 1
    if settings.resume {
        ctx.resume = true;
        ctx.loop_iteration = 1;
    }

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
                logging::tseprintln!(&settings, "Error executing command: {e}");
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

        peer::sort_keys(&mut ctx.missing_keys, &ctx.peers);
        peer::sort_keys(&mut ctx.late_keys, &ctx.peers);

        delta.compute_from(ctx);

        // --skip-first logic is here
        // Only skip after we've computed the delta
        if should_skip_next {
            if ctx.is_first_run() {
                // If you --skip-first the first run, reminds will never be sent
                // because the stateful notifiers will never have their
                // last_notification_sent set. So fake a notification being sent here, once.
                // The alternative is to keep a program_started_at timestamp
                // in Context and check against that in the reminder logic, but
                // this is simpler, leverages existing code and achieves the same results.
                // Hacky, though.
                for n in notifiers.iter_mut() {
                    n.state_mut().on_successful_notification(&ctx.now);
                }
            }

            should_skip_next = false;
            end_loop(ctx, time::Duration::ZERO);
            continue;
        }

        // !delta.is_empty() means "there was at least one change since the last loop"
        // which is another way of saying "there is at least one new notification to send".
        if !delta.is_empty() {
            if settings.debug {
                delta.print_nonempty_prefixed("... ");
            }

            let report = notify::send_notification(ctx, &delta, notifiers, &settings);

            if settings.debug && report.total != report.skipped {
                println!("{:#?}\n", report);
            }

            end_loop(ctx, settings.monitor.check_interval);
            continue;
        }

        if ctx.is_first_run() {
            let _ = notify::send_notification(ctx, &delta, notifiers, &settings);
            end_loop(ctx, settings.monitor.check_interval);
            continue;
        }

        // !ctx.missing_keys.is_empty() || !ctx.late_keys.is_empty() means
        // "there is at least one peer missing or late"
        if !ctx.missing_keys.is_empty() || !ctx.late_keys.is_empty() {
            let report = notify::send_reminder(ctx, notifiers, &settings);

            if settings.debug && report.total != report.skipped {
                println!("{:#?}\n", report);
            }
        }

        // Either there are no peers missing/late or there are but no
        // reminders were due, so check for pending notifications.
        let report = notify::retry_pending_notifications(notifiers, &settings);

        if settings.debug && report.total != report.skipped {
            println!("{:#?}\n", report);
        }

        end_loop(ctx, settings.monitor.check_interval);
    }
}

/// Initializes the program settings by loading configuration from the specified
/// configuration directory, applying any overrides from the command-line
/// arguments, and performing necessary validation and setup.
///
/// # Parameters
/// - `cli`: The parsed command-line arguments, used to determine the
/// configuration directory and any overrides to apply to the settings.
///
/// # Returns
/// On success, returns the initialized `Settings` object. On failure,
/// returns an appropriate exit code indicating the type of failure that
/// occurred during initialization.
fn init_settings(cli: &cli::Cli) -> Result<settings::Settings, process::ExitCode> {
    let mut settings = settings::Settings::default();

    if let Err(e) = settings.inherit_config_dir(&cli.config_dir) {
        logging::tseprintln!(
            &settings,
            "Error resolving default configuration directory: {e}"
        );
        return Err(process::ExitCode::from(
            defaults::exit_codes::FAILED_TO_RESOLVE_CONFIG_DIR,
        ));
    }

    if !settings.paths.config_dir.exists() && !cli.save {
        logging::tseprintln!(
            &settings,
            "Configuration directory {} does not exist. \
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
            logging::tseprintln!(
                &settings,
                "Failed to read configuration file {}: {e}",
                settings.paths.config_file.display()
            );
            return Err(process::ExitCode::from(
                defaults::exit_codes::FAILED_TO_READ_CONFIG_FILE,
            ));
        }
    };

    if !cli.save && config.is_none() {
        logging::tseprintln!(
            &settings,
            "No configuration file found at {}. \
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
                    logging::tsprintln!(
                        &settings,
                        "Configuration directory {} created.",
                        settings.paths.config_dir.display()
                    );
                }
                Err(e) => {
                    logging::tseprintln!(
                        &settings,
                        "Failed to create configuration directory {}: {e}",
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
            logging::tseprintln!(
                &settings,
                "Failed to write configuration file {}: {e}",
                settings.paths.config_file.display()
            );

            return Err(process::ExitCode::from(
                defaults::exit_codes::FAILED_TO_WRITE_CONFIG_FILE,
            ));
        };

        if !settings.paths.peer_list.exists() {
            match fs::write(&settings.paths.peer_list, defaults::EMPTY_PEER_LIST_CONTENT) {
                Ok(()) => {
                    logging::tsprintln!(
                        &settings,
                        "Empty peer list file {} created.",
                        settings.paths.peer_list.display()
                    );
                }
                Err(e) => {
                    logging::tseprintln!(
                        &settings,
                        "Failed to write empty peer list file {}: {e}",
                        settings.paths.peer_list.display()
                    );

                    return Err(process::ExitCode::from(
                        defaults::exit_codes::FAILED_TO_WRITE_PEER_LIST_FILE,
                    ));
                }
            };
        }

        logging::tsprintln!(
            &settings,
            "Configuration and resources written successfully to {}.",
            settings.paths.config_dir.display()
        );
        return Err(process::ExitCode::SUCCESS);
    }

    Ok(settings)
}
