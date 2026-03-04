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
        settings.print();
        return process::ExitCode::SUCCESS;
    }

    if let Err(sanity_check_failures) = settings.sanity_check() {
        eprintln!("Configuration has errors:");

        for error in sanity_check_failures {
            eprintln!("  * {error}");
        }

        if settings.dry_run {
            println!("Continuing anyway because --dry-run is set.");
            println!();
        } else {
            return process::ExitCode::from(defaults::exit_codes::CONFIGURATION_ERROR);
        }
    }

    let peers = match wireguard::read_peer_list(&settings.paths.peer_list, settings.debug) {
        Ok(peers) => peers,
        Err(e) => {
            eprintln!("Error reading peers file: {}", e);
            return process::ExitCode::from(defaults::exit_codes::ERROR_READING_PEERS_FILE);
        }
    };

    settings.print();
    println!();

    let latest_handshakes_output = match wireguard::get_handshakes(&settings.monitor.interface) {
        Ok(output) => {
            /*if settings.debug {
                println!("{output}");
            }*/
            output
        }
        Err(e) => {
            eprintln!("Error executing command: {e}");
            return process::ExitCode::from(
                defaults::exit_codes::FAILED_TO_EXECUTE_HANDSHAKES_COMMAND,
            );
        }
    };

    let handshake_validation_errors = wireguard::validate_handshakes(&latest_handshakes_output);

    if !handshake_validation_errors.is_empty() {
        eprintln!("Error validating latest-handshakes output:");

        for error in handshake_validation_errors {
            eprintln!("  * {error}");
        }

        if settings.dry_run {
            println!("Continuing anyway because --dry-run is set.");
            println!();
        } else {
            return process::ExitCode::from(
                defaults::exit_codes::FAILED_TO_PARSE_HANDSHAKES_OUTPUT,
            );
        }
    }

    let mut notifiers = build_notifiers(&settings);

    if notifiers.is_empty() {
        eprintln!("No notifiers configured.");

        if settings.dry_run {
            println!("Continuing anyway because --dry-run is set.");
            println!();
        } else {
            return process::ExitCode::from(defaults::exit_codes::NO_NOTIFIERS_CONFIGURED);
        }
    }

    let mut ctx = notify::Context::inherit(peers);
    run_loop(&mut ctx, &mut notifiers, settings)
}

/// Construct notifiers based on the settings, returning a vector of boxed trait objects.
fn build_notifiers(settings: &Settings) -> Vec<Box<dyn notify::NotificationSender>> {
    let mut notifiers: Vec<Box<dyn notify::NotificationSender>> = Vec::new();
    let client = Arc::new(blocking::Client::new());

    let build_slack_notifier = |i: usize, url: &str| {
        let slack_backend = backend::SlackBackend::new(
            i,
            Arc::clone(&client),
            url,
            &settings.slack.strings,
            &settings.slack.reminder_strings,
        );

        notify::Notifier::new(slack_backend, settings.dry_run)
    };

    let build_batsign_notifier = |i: usize, url: &str| {
        let batsign_backend = backend::BatsignBackend::new(
            i,
            Arc::clone(&client),
            url,
            &settings.batsign.strings,
            &settings.batsign.reminder_strings,
        );

        notify::Notifier::new(batsign_backend, settings.dry_run)
    };

    if settings.dry_run {
        notifiers.push(Box::new(build_slack_notifier(0, defaults::DUMMY_SLACK_URL)));
        notifiers.push(Box::new(build_batsign_notifier(
            0,
            defaults::DUMMY_BATSIGN_URL,
        )));
    } else {
        if settings.slack.enabled && !settings.slack.urls.is_empty() {
            for (i, url) in settings.slack.urls.iter().enumerate() {
                notifiers.push(Box::new(build_slack_notifier(i, url)));
            }
        }

        if settings.batsign.enabled && !settings.batsign.urls.is_empty() {
            for (i, url) in settings.batsign.urls.iter().enumerate() {
                notifiers.push(Box::new(build_batsign_notifier(i, url)));
            }
        }
    }

    notifiers
}

/// Main loop of the program.
fn run_loop(
    ctx: &mut notify::Context,
    notifiers: &mut [Box<dyn notify::NotificationSender>],
    settings: Settings,
) -> process::ExitCode {
    let mut delta = notify::Delta::with_capacity(ctx.peers.len());

    ctx.first_run = true;

    loop {
        match wireguard::get_handshakes(&settings.monitor.interface) {
            Ok(output) => {
                if settings.debug {
                    println!("{output}");
                }
                wireguard::update_handshakes(&output, &mut ctx.peers);
            }
            Err(e) => {
                eprintln!("Error executing command: {e}");
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
                            "Peer '{}': last seen {} seconds ago at {}",
                            peer.human_name,
                            age.as_secs(),
                            dt.format("%Y-%m-%d %H:%M:%S")
                        );
                    }

                    if age < settings.monitor.timeout {
                        continue;
                    }

                    if settings.debug {
                        println!("age is greater than timeout, marking as late");
                    }

                    ctx.late_keys.push(key.clone());
                }
                None => {
                    if settings.debug {
                        println!(
                            "Peer '{}' has never been seen, marking as missing",
                            peer.human_name
                        );
                    }

                    ctx.missing_keys.push(key.clone());
                }
            }
        }

        delta.compute_from(ctx);

        if delta.is_empty() {
            if ctx.missing_keys.is_empty() && ctx.late_keys.is_empty() {
                ctx.rotate();
                ctx.first_run = false;
                thread::sleep(settings.monitor.check_interval);
                continue;
            }

            if let Some(last_report_timestamp) = ctx.last_report
                && ctx
                    .now
                    .duration_since(last_report_timestamp)
                    .unwrap_or(time::Duration::ZERO)
                    > settings.monitor.reminder_interval
            {
                match notify::send_reminder(notifiers, ctx) {
                    true => {
                        println!("Repeat reminders sent successfully");
                        ctx.last_report = Some(ctx.now);
                    }
                    false => eprintln!("Failed to send some repeat reminders"),
                }

                println!();
            }

            ctx.rotate();
            ctx.first_run = false;
            thread::sleep(settings.monitor.check_interval);
            continue;
        }

        if settings.debug {
            if !delta.no_longer_late_keys.is_empty() {
                println!("no_longer_late_keys:    {:#?}", delta.no_longer_late_keys);
            }
            if !delta.became_late_keys.is_empty() {
                println!("became_late_keys:     {:#?}", delta.became_late_keys);
            }
            if !delta.returned_keys.is_empty() {
                println!("returned_keys: {:#?}", delta.returned_keys);
            }
            if !delta.went_missing_keys.is_empty() {
                println!("went_missing_keys:  {:#?}", delta.went_missing_keys);
            }
        }

        match notify::send_notification(notifiers, ctx, &delta) {
            true => {
                println!("Notifications sent successfully");
            }
            false => {
                // We can either drop down here, rotate hashmaps and unset first_run
                // so as to force a retry of the same notification after sleeping,
                // or we can just log the failure and move on.
                // For now, just log. Failed notifications will be swallowed.
                eprintln!("Failed to send some notifications");
                /*thread::sleep(defaults::LOOP_INTERVAL);
                continue;*/
            }
        }

        println!();

        ctx.rotate();
        ctx.first_run = false;
        ctx.last_report = Some(ctx.now);
        thread::sleep(settings.monitor.check_interval);
    }
}

/// Initializes all settings, except for CLI parsing, which must already have been done.
fn init_settings(cli: &cli::Cli) -> Result<Settings, process::ExitCode> {
    let mut settings = Settings::default();

    if let Err(e) = settings.inherit_config_dir(&cli.config_dir) {
        eprintln!("Error resolving default configuration directory: {}", e);
        return Err(process::ExitCode::from(
            defaults::exit_codes::FAILED_TO_RESOLVE_CONFIG_DIR,
        ));
    }

    if !settings.paths.config_dir.exists() && !cli.save {
        eprintln!(
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
            eprintln!(
                "Failed to read configuration file {}: {e}",
                settings.paths.config_file.display()
            );
            return Err(process::ExitCode::from(
                defaults::exit_codes::FAILED_TO_READ_CONFIG_FILE,
            ));
        }
    };

    if !cli.save && config.is_none() {
        eprintln!(
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
                    println!(
                        "Configuration directory {} created.",
                        settings.paths.config_dir.display()
                    );
                }
                Err(e) => {
                    eprintln!(
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
            eprintln!(
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
                    println!(
                        "Empty peer list file {} created.",
                        settings.paths.peer_list.display()
                    );
                }
                Err(e) => {
                    eprintln!(
                        "Failed to write empty peer list file {}: {e}",
                        &settings.paths.peer_list.display()
                    );

                    return Err(process::ExitCode::from(
                        defaults::exit_codes::FAILED_TO_WRITE_PEER_LIST_FILE,
                    ));
                }
            };
        }

        println!(
            "Configuration and resources written successfully to {}.",
            settings.paths.config_dir.display()
        );
        return Err(process::ExitCode::SUCCESS);
    }

    Ok(settings)
}
