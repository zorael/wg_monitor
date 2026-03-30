//! Monitors other peers in a [**WireGuard**](https://www.wireguard.com) VPN
//! and sends a notification if contact with a peer is lost.
//!
//! The main purpose of this is to monitor Internet-connected locations for
//! power outages, using WireGuard handshakes as a way for sites to phone home.
//! Each site needs an always-on, always-online computer to act as a WireGuard
//! peer, for which something like a
//! [Raspberry Pi Zero 2W](https://www.raspberrypi.com/products/raspberry-pi-zero-2-w)
//! is cheap and more than sufficient. (May require cross-compilation.)
//!
//! In a hub-and-spoke WireGuard configuration, this should be run on the hub
//! server, with an additional instance on at least one other
//! geographically disconnected peer to monitor the hub. In other configurations,
//! it can be run on any peer with visibility of other peers, but a secondary
//! instance monitoring the first is recommended in any setup.
//! If the hub loses power, it cannot report itself as being lost.
//!
//! Peers must have a `PersistentKeepalive` setting in their WireGuard
//! configuration with a value *comfortably lower* than the peer timeout of
//! this program. This timeout is **10 minutes** by default.
//!
//! Notifications can be sent as
//! [**Slack**](https://docs.slack.dev/messaging/sending-messages-using-incoming-webhooks)
//! messages, as short emails via [**Batsign**](https://batsign.me), and/or by
//! invocation of an [**external command**](#external-command)
//! (like `notify-send`, `wall` or `sendmail`).
//!
//! At any given time, any given peer is in one of three states:
//!
//! - **present**: the peer has been seen within the timeout period.
//! - **lost**: the peer has been seen before but has not been seen within the
//!   timeout period. It may be referred to as "lost" in some message strings.
//! - **missing**: the peer has not been seen since the last restart of the VPN.
//!
//! As such, peers may be in the following transition states;
//!
//! - **now lost**: the peer was present but is now lost.
//! - **now missing**: the peer was present but is now missing, which is usually
//!   indicative of a restart of the VPN. This may be referred to as "lost due
//!   to a network reset" in some message strings.
//! - **was lost**: the peer was lost but is now present again. This may
//!   be referred to as "returned" in some message strings.
//! - **was missing**: the peer was missing (had never been seen) but is now present.
//!   This may be referred to as "appeared" in some message strings.

mod backend;
mod cli;
mod defaults;
mod file_config;
mod logging;
mod notify;
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
        "{} v{} | copyright (c) 2026 {}\n$ git clone {}",
        defaults::program_metadata::PROGRAM_NAME,
        defaults::program_metadata::VERSION,
        defaults::program_metadata::AUTHORS,
        defaults::program_metadata::SOURCE_REPOSITORY
    );
}

/// Main entrypoint of the program.
fn main() -> process::ExitCode {
    let cli = cli::Cli::parse();

    print_banner();
    println!();

    if cli.version {
        // This is the only way to get a neat --version output.
        // The banner with version is already printed just prior to this,
        // so we can just exit successfully here after echoing the licenses.
        println!(
            "This project is dual-licensed under the MIT License and \
            the Apache License (Version 2.0) at your option."
        );
        return process::ExitCode::SUCCESS;
    }

    let settings = match init_settings(&cli) {
        Outcome::Success(s) => *s, // dereference to move out of the Box
        Outcome::EarlyExitCode(code) => return code,
    };

    if cli.show {
        // If --show was passed, print settings here and exit early.
        settings.print();
        return process::ExitCode::SUCCESS;
    }

    if let Err(settings_sanity_check_failures) = settings.sanity_check() {
        logging::tseprintln!(
            &settings.disable_timestamps,
            "Incomplete or invalid configuration:"
        );

        for error in settings_sanity_check_failures {
            eprintln!("  - {error}");
        }

        if settings.dry_run {
            logging::tsprintln!(
                &settings.disable_timestamps,
                "Continuing anyway because --dry-run is set."
            );
        } else {
            return process::ExitCode::from(defaults::exit_codes::CONFIGURATION_ERROR);
        }
    }

    let peers = match wireguard::read_peer_list(&settings.paths.peer_list, settings.debug) {
        Ok(peers) if peers.is_empty() => {
            logging::tseprintln!(
                &settings.disable_timestamps,
                "Peer list file {} is empty.",
                settings.paths.peer_list.display()
            );
            return process::ExitCode::from(defaults::exit_codes::EMPTY_PEER_LIST);
        }
        Ok(peers) => peers,
        Err(e) => {
            logging::tseprintln!(
                &settings.disable_timestamps,
                "Error reading peers file: {e}"
            );
            return process::ExitCode::from(defaults::exit_codes::ERROR_READING_PEERS_FILE);
        }
    };

    let mut notifiers = build_notifiers(&settings);

    if notifiers.is_empty() {
        logging::tseprintln!(&settings.disable_timestamps, "No notifiers configured.");

        if settings.dry_run {
            logging::tsprintln!(
                &settings.disable_timestamps,
                "Continuing anyway because --dry-run is set.",
            );
            println!();
        } else {
            return process::ExitCode::from(defaults::exit_codes::NO_NOTIFIERS_CONFIGURED);
        }
    }

    if let Err(notifier_sanity_check_failures) = sanity_check_notifiers(&notifiers) {
        logging::tseprintln!(
            &settings.disable_timestamps,
            "Incomplete or invalid notifier configuration:"
        );

        for error in notifier_sanity_check_failures {
            eprintln!("  - {error}");
        }

        if settings.dry_run {
            logging::tsprintln!(
                &settings.disable_timestamps,
                "Continuing anyway because --dry-run is set.",
            );
            println!();
        } else {
            return process::ExitCode::from(defaults::exit_codes::INVALID_NOTIFIER_CONFIGURATION);
        }
    }

    // Sleep if --sleep was passed, to allow for the interface to come up
    // and/or to allow for peers to be seen before we do the initial handshake check.
    if let Some(duration) = cli.sleep
        && duration > time::Duration::from_secs(0)
    {
        logging::tsprintln!(
            &settings.disable_timestamps,
            "Sleeping for {} before starting monitoring loop...",
            humantime::format_duration(duration)
        );

        thread::sleep(duration);

        logging::tsprintln!(
            &settings.disable_timestamps,
            "Finished sleep. Continuing..."
        );
    }

    // Verify that we can execute the `wg show` command but don't actually care
    // about the handshakes at this point. We just want to verify that the
    // command executes successfully before entering the main loop.
    let latest_handshakes_output = match get_first_handshakes_output(&settings) {
        Outcome::Success(output) => output,
        Outcome::EarlyExitCode(code) => return code,
    };

    // Likewise verify that the output of `wg show {iface} latest-handshakes`
    // doesn't have anything unexpected in it before starting the main loop.
    if let Err(handshake_validation_errors) =
        wireguard::validate_handshakes(&latest_handshakes_output)
    {
        logging::tseprintln!(
            &settings.disable_timestamps,
            "Error validating latest-handshakes output:",
        );

        for error in handshake_validation_errors {
            eprintln!("  - {error}");
        }

        if settings.dry_run {
            logging::tsprintln!(
                &settings.disable_timestamps,
                "Continuing anyway because --dry-run is set.",
            );
            println!();
        } else {
            return process::ExitCode::from(
                defaults::exit_codes::FAILED_TO_PARSE_HANDSHAKES_OUTPUT,
            );
        }
    }

    logging::tsprintln!(&settings.disable_timestamps, "Initialization complete.");

    if settings.debug {
        println!();
        settings.print();
        println!();
        println!("{:#?}", peers);
    }

    if settings.verbose || settings.debug {
        println!();
        println!(
            "{} {} monitored, {} {} configured.",
            peers.len(),
            utils::plurality(peers.len() as isize, "peer", "peers"),
            notifiers.len(),
            utils::plurality(notifiers.len() as isize, "notifier", "notifiers")
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
    }

    if settings.dry_run {
        logging::tsprintln!(&settings.disable_timestamps, "DRY RUN");
    }

    // All done, create the initial context
    let mut ctx = notify::Context::inherit(peers);

    // Store the peer list file path in the context so that backends can access it
    ctx.peer_list = settings.paths.peer_list.clone();

    // And finally enter the loop.
    logging::tsprintln!(&settings.disable_timestamps, "Entering main loop...");
    run_loop(&mut ctx, &mut notifiers, settings)
}

/// Initializes the program settings by loading configuration from the specified
/// configuration directory, applying any overrides from the command-line
/// arguments, and performing necessary validation and setup.
///
/// # Parameters
/// - `cli`: The parsed command-line arguments, used to determine the
///   configuration directory and any overrides to apply to the settings.
///
/// # Returns
/// An `InitSettingsResult` which is either:
/// - `Success` containing the initialized `Settings` instance, boxed in a
///   `Box<settings::Settings>` for memory reasons
/// - `EarlyExitCode` containing a `process::ExitCode` to exit with if
///   initialization fails, alternatively if the `--save` flag was passed to
///   generate configuration and resources.
fn init_settings(cli: &cli::Cli) -> Outcome<Box<settings::Settings>> {
    let mut settings = settings::Settings::default();

    if let Err(e) = settings.inherit_config_dir(&cli.config_dir) {
        logging::tseprintln!(
            &settings.disable_timestamps,
            "Error resolving default configuration directory: {e}"
        );
        return Outcome::EarlyExitCode(process::ExitCode::from(
            defaults::exit_codes::FAILED_TO_RESOLVE_CONFIG_DIR,
        ));
    }

    // Error out if the configuration directory doesn't exist *unless* --save
    // was passed, in which case we'll create it later.
    if !settings.paths.config_dir.exists() && !cli.save {
        logging::tseprintln!(
            &settings.disable_timestamps,
            "Configuration directory {} does not exist. \
            Create it or run with `--save` to generate default configuration and resources.",
            settings.paths.config_dir.display()
        );
        return Outcome::EarlyExitCode(process::ExitCode::from(
            defaults::exit_codes::CONFIG_DIR_DOES_NOT_EXIST,
        ));
    }

    settings.resolve_resource_paths();

    let config = match file_config::deserialize_config_file(&settings.paths.config_file) {
        Ok(cfg) => cfg,
        Err(e) => {
            // The configuration file exists but could not be read. Permissions issue?
            logging::tseprintln!(
                &settings.disable_timestamps,
                "Failed to read configuration file {}: {e}",
                settings.paths.config_file.display()
            );
            return Outcome::EarlyExitCode(process::ExitCode::from(
                defaults::exit_codes::FAILED_TO_READ_CONFIG_FILE,
            ));
        }
    };

    // No configuration file was found.
    // Error out unless --save was passed, in which case we'll create it later.
    if config.is_none() && !cli.save {
        logging::tseprintln!(
            &settings.disable_timestamps,
            "No configuration file found at {}. \
            Create it or run with `--save` to generate default configuration and resources.",
            settings.paths.config_file.display()
        );
        return Outcome::EarlyExitCode(process::ExitCode::from(
            defaults::exit_codes::CONFIG_FILE_DOES_NOT_EXIST,
        ));
    }

    settings.apply_file(&config);
    settings.apply_cli(cli);
    settings.clean_up();

    if cli.save {
        // If --save was passed, save the settings to the configuration file and
        // create an empty peer list file if they don't already exist.
        return Outcome::EarlyExitCode(save_settings_to_config_file(&settings));
    }

    // This need not be part of the --save routine to place it after where it returns
    settings.resolve_wg();

    // Box the resulting settings to avoid issues with the size of the Settings struct
    // making the InitSettingsResult enum too large to compile.
    Outcome::Success(Box::new(settings))
}

/// Generic return type for functions that initialize settings or perform
/// similar operations where the outcome can either be a successful result
/// or an early exit with a specific exit code.
enum Outcome<T> {
    /// Indicates a successful outcome, containing a value of type `T`.
    Success(T),

    /// Indicates that the operation should exit early with the provided
    /// `process::ExitCode`.
    ///
    /// This may be `process::SUCCESS` and is such not necessarily
    /// an error exit code.
    EarlyExitCode(process::ExitCode),
}

/// Gets the output of the `wg show {iface} latest-handshakes` command, retrying
/// as needed until it succeeds, and returns the output as an `Outcome<String>`.
///
/// # Parameters
/// - `settings`: The program settings, used to determine the path to the
///   `wg` command, the interface to monitor, and the check interval for
///   retrying if the command fails.
///
/// # Returns
/// - `Outcome::Success(String)` containing the output of the command if it
///   executes successfully.
/// - `Outcome::EarlyExitCode(process::ExitCode)` if the command fails to
///   execute due to what seems to not be a transient issue.
fn get_first_handshakes_output(settings: &settings::Settings) -> Outcome<String> {
    loop {
        match wireguard::get_handshakes(&settings.paths.wg, &settings.monitor.interface) {
            Ok(output) => break Outcome::Success(output),
            Err(e) => {
                let e = e.to_string();
                logging::tseprintln!(&settings.disable_timestamps, "{e}");

                if e.contains("No such device") {
                    logging::tsprintln!(
                        &settings.disable_timestamps,
                        "Interface {} down? Retrying in {}...",
                        settings.monitor.interface,
                        humantime::format_duration(settings.monitor.check_interval)
                    );

                    // Interface may not be up yet, such as if systemd is starting
                    // this program before the network is fully up.
                    thread::sleep(settings.monitor.check_interval);
                    continue;
                } else if e.contains("Operation not permitted") {
                    logging::tseprintln!(
                        &settings.disable_timestamps,
                        "Insufficient privileges to execute 'wg show' command."
                    );
                    return Outcome::EarlyExitCode(process::ExitCode::from(
                        defaults::exit_codes::INSUFFICIENT_PRIVILEGES,
                    ));
                } else {
                    logging::tseprintln!(
                        &settings.disable_timestamps,
                        "Failed to execute handshakes command."
                    );
                    return Outcome::EarlyExitCode(process::ExitCode::from(
                        defaults::exit_codes::FAILED_TO_EXECUTE_HANDSHAKES_COMMAND,
                    ));
                }
            }
        }
    }
}

/// Saves the provided settings to the configuration file and creates an empty
/// peer list file if one does not already exist.
///
/// # Notes
/// Refer to the `defaults::exit_codes` module for the specific exit codes used.
///
/// # Parameters
/// - `settings`: The program settings to save to the configuration file and use
///   for determining the paths to save to.
///
/// # Returns
/// A `process::ExitCode` indicating the result of the operation.
fn save_settings_to_config_file(settings: &settings::Settings) -> process::ExitCode {
    if !settings.paths.config_dir.exists() {
        match fs::create_dir_all(&settings.paths.config_dir) {
            Ok(()) => {
                logging::tsprintln!(
                    &settings.disable_timestamps,
                    "Configuration directory {} created.",
                    settings.paths.config_dir.display()
                );
            }
            Err(e) => {
                logging::tseprintln!(
                    &settings.disable_timestamps,
                    "Failed to create configuration directory {}: {e}",
                    settings.paths.config_dir.display()
                );

                return process::ExitCode::from(defaults::exit_codes::FAILED_TO_CREATE_CONFIG_DIR);
            }
        };
    }

    let config = file_config::FileConfig::from(settings);

    if let Err(e) = confy::store_path(&settings.paths.config_file, config) {
        logging::tseprintln!(
            &settings.disable_timestamps,
            "Failed to write configuration file {}: {e}",
            settings.paths.config_file.display()
        );

        return process::ExitCode::from(defaults::exit_codes::FAILED_TO_WRITE_CONFIG_FILE);
    };

    if !settings.paths.peer_list.exists() {
        match fs::write(&settings.paths.peer_list, defaults::EMPTY_PEER_LIST_CONTENT) {
            Ok(()) => {
                logging::tsprintln!(
                    &settings.disable_timestamps,
                    "Empty peer list file {} created.",
                    settings.paths.peer_list.display()
                );
            }
            Err(e) => {
                logging::tseprintln!(
                    &settings.disable_timestamps,
                    "Failed to write empty peer list file {}: {e}",
                    settings.paths.peer_list.display()
                );

                return process::ExitCode::from(
                    defaults::exit_codes::FAILED_TO_WRITE_PEER_LIST_FILE,
                );
            }
        };
    }

    logging::tsprintln!(
        &settings.disable_timestamps,
        "Configuration and resources written successfully to {}.",
        settings.paths.config_dir.display()
    );

    process::ExitCode::SUCCESS
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

    // Helper closure to build a Slack backend instance.
    let make_slack_backend = |i: usize, url: &str| {
        backend::SlackBackend::new(
            i,
            agent.clone(),
            url,
            settings.slack.show_response,
            &settings.slack.strings,
            &settings.slack.reminder_strings,
        )
    };

    // Helper closure to build a Batsign backend instance.
    let make_batsign_backend = |i: usize, url: &str| {
        backend::BatsignBackend::new(
            i,
            agent.clone(),
            url,
            settings.batsign.show_response,
            &settings.batsign.strings,
            &settings.batsign.reminder_strings,
        )
    };

    // Helper closure to build a Command backend instance.
    let make_command_backend = |i: usize, command: &str| {
        backend::CommandBackend::new(
            i,
            command,
            settings.command.show_output,
            &settings.command.strings,
            &settings.command.reminder_strings,
        )
    };

    let (slack_enabled, batsign_enabled, command_enabled) = match (
        settings.slack.enabled,
        settings.batsign.enabled,
        settings.command.enabled,
    ) {
        (false, false, false) if settings.dry_run => {
            logging::tseprintln!(
                &settings.disable_timestamps,
                "No backends enabled. Enabling all backends because --dry-run is set."
            );
            (true, true, true)
        }
        other => other,
    };

    // Use dummy URLs for dry runs so that we can get output for all (enabled)
    // backends even if no URLs were configured.

    let slack_urls = match settings.dry_run {
        true if slack_enabled => vec![defaults::DUMMY_SLACK_URL.to_string()],
        _ => settings.slack.urls.clone(),
    };

    let batsign_urls = match settings.dry_run {
        true if batsign_enabled => vec![defaults::DUMMY_BATSIGN_URL.to_string()],
        _ => settings.batsign.urls.clone(),
    };

    let commands = match settings.dry_run {
        true if command_enabled => vec![defaults::DUMMY_COMMAND.to_string()],
        _ => settings.command.commands.clone(),
    };

    if slack_enabled && !slack_urls.is_empty() {
        build_and_push_notifiers(
            &mut notifiers,
            &slack_urls,
            make_slack_backend,
            settings.dry_run,
        );
    }

    if batsign_enabled && !batsign_urls.is_empty() {
        build_and_push_notifiers(
            &mut notifiers,
            &batsign_urls,
            make_batsign_backend,
            settings.dry_run,
        )
    }

    if command_enabled && !commands.is_empty() {
        build_and_push_notifiers(
            &mut notifiers,
            &commands,
            make_command_backend,
            settings.dry_run,
        )
    }

    notifiers
}

/// Helper function to build and push notifiers for a passed backend type into a
/// passed vector.
///
/// This is only called from within `build_notifiers`, but as
/// it doesn't actually use any variables from that scope, it can be a
/// standalone function.
///
/// This function iterates over the provided elements (URLs or commands), uses
/// the provided `make_backend_fn` to create backend instances for each element,
/// wraps them in `Notifier` instances, and pushes them into the provided vector
/// of notifiers. The `dry_run` parameter is passed to the `Notifier`
/// constructor to allow for appropriate behavior in dry-run mode.
///
/// # Parameters
/// - `vec`: The mutable vector of boxed `StatefulNotifier` trait objects to push
///   the new notifiers into.
/// - `elements`: A slice of strings representing the configuration elements for
///   the backend (such as URLs for Slack/Batsign or commands for Command backend).
/// - `make_backend_fn`: A closure that takes an index and a reference to a
///   string and returns an instance of the backend type `B`.
/// - `dry_run`: A boolean indicating whether the notifiers being created are
///   for dry-run mode, which may affect how the notifiers behave when sending
///   notifications.
fn build_and_push_notifiers<B, F>(
    vec: &mut Vec<Box<dyn notify::StatefulNotifier>>,
    elements: &[String],
    mut make_backend_fn: F,
    dry_run: bool,
) where
    B: backend::Backend + 'static,
    F: FnMut(usize, &str) -> B,
{
    for (i, element) in elements.iter().enumerate() {
        let backend = make_backend_fn(i, element);
        let boxed = Box::new(notify::Notifier::new(backend, dry_run));
        vec.push(boxed);
    }
}

/// Performs a sanity check on all notifiers, validating their backends' settings.
///
/// If any issues are found, a vector of descriptive error messages is returned.
///
/// # Parameters
/// - `notifiers`: A slice of boxed `StatefulNotifier` trait objects to check.
///
/// # Returns
/// - `Ok(())` if all notifiers passed their sanity checks without any issues.
/// - `Err(Vec<String>)` if there were issues found during the sanity checks.
fn sanity_check_notifiers(
    notifiers: &[Box<dyn notify::StatefulNotifier>],
) -> Result<(), Vec<String>> {
    let mut vec = Vec::new();

    for notifier in notifiers.iter() {
        if let Err(mut errors) = notifier.sanity_check() {
            vec.append(&mut errors);
        }
    }

    if vec.is_empty() { Ok(()) } else { Err(vec) }
}

/// Runs the main monitoring loop, which continuously checks the status of peers
/// and sends notifications as needed.
///
/// This function will run indefinitely until the program is terminated. It performs
/// the following steps in each iteration:
///
/// 1. Executes the `wg show` command to get the latest handshakes and updates
///    the context with the new information.
/// 2. Calculates a `notify::KeyDelta` that represents the difference between
///    the current peer context (who is present, lost, missing) and the previous one.
/// 3. If this is the first run, sends a notification with the initial state of all peers.
/// 4. If there are any notifiers with pending failures, attempts to retry them.
///    The number of remaining failures after the retry attempt is recorded.
///    (If this number is 0, all retries have succeeded.)
///    The loop proceeds; there are no `continue`s after this step.
/// 5. If there were any changes between the previous loop and the current one,
///    determined by whether or not the `notify::KeyDelta` is empty, it infers
///    that there should be a notification sent. Assuming one was not sent too
///    recently, it pushes one about these changes through all notifiers.
///    `end_loop` is called here and the loop continues to the next iteration.
/// 6. If there were no changes but there are still lost or missing peers,
///    it infers there should be a remind sent. Assuming one was not sent too
///    recently, it pushes a reminder through all notifiers.
///    `end_loop` is called here and the loop continues to the next iteration.
/// 7. If there were no changes and no lost/missing peers, it infers that there
///    is nothing to notify about and does not send anything.
///    `end_loop_minimal` is called here and the loop continues to the next iteration.
///
/// # Parameters
/// - `ctx`: The notification context, which holds the current state of peers
///   and other relevant information.
/// - `notifiers`: A mutable slice of stateful notifiers to use for
///   sending notifications.
/// - `settings`: The program settings, used to determine behavior such as
///   intervals and debug output.
///
/// # Returns
/// This function will never return under normal operation, as it runs an
/// infinite loop. It will only return an exit code if the loop is somehow
/// exited, which indicates a shutdown or critical failure.
/// In normal operation, the program should be terminated externally
/// (via a signal) rather than exiting this function.
fn run_loop(
    ctx: &mut notify::Context,
    notifiers: &mut [Box<dyn notify::StatefulNotifier>],
    settings: settings::Settings,
) -> process::ExitCode {
    let mut should_skip_next = settings.skip_first;

    // If `resume` is set, we want to skip the first run. The easiest way is to
    // just set start `loop_iteration` at 1
    if settings.resume {
        ctx.resume = true;
        ctx.loop_iteration = 1;
    }

    // Keep a copy of the previous context to compute deltas against.
    let previous_ctx = &mut ctx.clone();

    // Add a linebreak in debug mode for better spacing
    if settings.debug {
        println!();
    }

    loop {
        match wireguard::get_handshakes(&settings.paths.wg, &settings.monitor.interface) {
            Ok(output) => {
                if settings.debug {
                    // This is very spammy so gate it behind debug instead of verbose mode.
                    println!("{output}");
                }
                wireguard::update_handshakes(&output, &mut ctx.peers);
            }
            Err(e) => {
                logging::tseprintln!(&settings.disable_timestamps, "Error executing command: {e}");
                thread::sleep(settings.monitor.check_interval);
                continue;
            }
        };

        if settings.debug {
            // Add a separator line between loop iterations
            logging::tsprintln!(
                &settings.disable_timestamps,
                "{}\n-------------------------------------------------------------",
                ctx.loop_iteration
            );
            println!();
        }

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
                            "  - Peer '{}': last seen {} seconds ago at {}",
                            peer.human_name,
                            age.as_secs(),
                            dt.format("%Y-%m-%d %H:%M:%S")
                        );
                    }

                    if age < settings.monitor.timeout {
                        continue;
                    }

                    if settings.debug {
                        println!("... age is greater than timeout, marking as lost");
                    }

                    ctx.lost_keys.push(key.clone());
                }
                None => {
                    if settings.debug {
                        println!(
                            "  - Peer '{}' has never been seen, marking as missing",
                            peer.human_name
                        );
                    }

                    ctx.missing_keys.push(key.clone());
                }
            }
        }

        // --skip-first logic is here
        if should_skip_next {
            if ctx.is_first_run() {
                // If you --skip-first the first run, reminders will never be sent
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
            end_loop_minimal(ctx, previous_ctx);
            thread::sleep(time::Duration::ZERO);
            continue;
        }

        // Sort keys by how long they've been lost.
        // Don't sort missing keys. None of them should have ever been seen,
        // so there's no meaningful way to sort them.
        wireguard::sort_keys(&mut ctx.lost_keys, &ctx.peers);

        let delta = notify::Context::delta_between(ctx, previous_ctx);

        if ctx.is_first_run() {
            let first_run_report = notify::send_notification(ctx, &delta, notifiers, &settings);
            end_loop(ctx, previous_ctx, first_run_report, &settings);
            continue;
        }

        let mut num_notifiers_with_failures = notifiers
            .iter()
            .filter(|n| n.state().failed_ctx.is_some())
            .count() as u32;

        if num_notifiers_with_failures > 0 {
            let retry_report = notify::retry_pending_notifications(ctx, notifiers, &settings);

            if settings.debug && retry_report.total != retry_report.skipped {
                println!("{:#?}\n", retry_report);
            }

            // At this point a retry has been attempted based on the current
            // context stored as "failing" in each notifier.
            // Some of those retries may have succeeded, some may have failed
            // again, and some may have been skipped because they were rate-limited.
            // Record how many notifiers still have failures after this retry
            // attempt, so we can use the information later when it's time to
            // decide how long to sleep.
            num_notifiers_with_failures -= retry_report.successful;

            if retry_report.successful > 0 || retry_report.failed > 0 {
                // One or more retry attempts were made and either succeeded, or failed.
                // The important part here is that attempts *were* made, so in the
                // case where there are more notifications waiting below,
                // we want to sleep a bit to rate-limit ourselves slightly.
                // The number is just a guess at a reasonable amount.
                thread::sleep(time::Duration::from_secs(5));
            }
        }

        let there_was_at_least_one_change_since_previous_loop = !delta.is_empty();

        if there_was_at_least_one_change_since_previous_loop {
            if settings.debug {
                delta.print_nonempty_keys_prefixed("... ");
            }

            let notification_report = notify::send_notification(ctx, &delta, notifiers, &settings);

            if settings.debug && notification_report.total != notification_report.skipped {
                println!();
                println!("{:#?}\n", notification_report);
            }

            end_loop(ctx, previous_ctx, notification_report, &settings);
            continue;
        }

        // If we're here, there were no changes since the previous loop

        let there_is_at_least_one_peer_missing_or_lost =
            !ctx.missing_keys.is_empty() || !ctx.lost_keys.is_empty();

        if there_is_at_least_one_peer_missing_or_lost {
            let reminder_report = notify::send_reminder(ctx, notifiers, &settings);

            if settings.debug && reminder_report.total != reminder_report.skipped {
                println!();
                println!("{:#?}\n", reminder_report);
            }

            end_loop(ctx, previous_ctx, reminder_report, &settings);
            continue;
        }

        end_loop_minimal(ctx, previous_ctx);

        if num_notifiers_with_failures > 0 {
            thread::sleep(settings.monitor.retry_interval);
        } else {
            thread::sleep(settings.monitor.check_interval);
        }
    }
}

/// Perform some cleanup at the end of a loop.
///
/// This can be called directly if the callsite takes care of sleeping, but is
/// otherwise mostly called indirectly via `end_loop`.
///
/// It has three main responsibilities;
///
/// 1. Rotate the current `notify::Context` into the previous one and clear itself
///    afterwards, making it a clean slate for the next loop iteration.
/// 2. Increment the loop iteration count in the context.
/// 3. Set the `resume` flag to `false`, since if this function is being called,
///    we're no longer in the initial state in which `resume` plays a role.
///
/// # Parameters
/// - `ctx`: The `notify::Context` used as a basis for the notification attempt.
/// - `previous_ctx`: The previous `notify::Context` from the previous loop
///   iteration, which will be overwritten with the current context's data.
fn end_loop_minimal(ctx: &mut notify::Context, previous_ctx: &mut notify::Context) {
    ctx.rotate_into(previous_ctx);
    ctx.loop_iteration = previous_ctx.loop_iteration + 1;
    ctx.resume = false;
}

/// Performs some clean-up and sleeps depending on the results of a
/// notification attempt.
///
/// `end_loop_minimal` is used for the clean-up. The additional logic in this
/// function is to determine how long to sleep for.
///
/// # Parameters
/// - `ctx`: The `notify::Context` used as a basis for the notification attempt.
/// - `previous_ctx`: The previous `notify::Context` from the previous loop iteration.
/// - `report`: The report from the notification attempt.
/// - `settings`: The program settings, which houses the configured intervals
///   for sleeping after loops.
fn end_loop(
    ctx: &mut notify::Context,
    previous_ctx: &mut notify::Context,
    report: notify::DispatchReport,
    settings: &settings::Settings,
) {
    end_loop_minimal(ctx, previous_ctx);

    if report.failed > 0 {
        thread::sleep(settings.monitor.retry_interval)
    } else {
        thread::sleep(settings.monitor.check_interval)
    }
}
