//! Types related to the root settings struct, which aggregates all other
//! settings types.
//!
//! This module defines the `Settings` struct, which serves as the root
//! configuration struct for the application, aggregating all other settings
//! types such as `MonitorSettings`, `SlackSettings`, `BatsignSettings`,
//! `CommandSettings`, and `PathBufs`.

use std::path;

use crate::cli;
use crate::defaults;
use crate::file_config;
use crate::wireguard;

/// Root settings struct that aggregates all other settings types.
///
/// This struct serves as the main settings struct for the application, holding
/// all runtime settings that are used throughout the program's operation.
#[derive(Debug, Default)]
pub struct Settings {
    /// Monitor settings for monitoring the WireGuard interface and connection status.
    pub monitor: super::MonitorSettings,

    /// Slack settings for sending notifications to Slack channels.
    pub slack: super::SlackSettings,

    /// Batsign settings for sending notifications via Batsign
    pub batsign: super::BatsignSettings,

    /// Command settings for executing external commands as notifications.
    pub command: super::CommandSettings,

    /// Paths to resources used by the program, such as the configuration
    /// directory, configuration file, and peer list file.
    pub paths: super::PathBufs,

    /// Whether to treat the first run loop of the program as resuming from a
    /// previous one, which affects how the first-run notification is worded.
    pub resume: bool,

    /// Whether to skip sending notifications for the first detected peer status
    /// change after the program starts, useful for avoiding sending
    /// notifications about the initial state.
    pub skip_first: bool,

    /// Whether to disable timestamps in notifications, useful for cases where
    /// the terminal output is routed into the systemd journal, where all lines
    /// are already timestamped -- or if the user simply prefers not to have
    /// timestamps in the notifications.
    pub disable_timestamps: bool,

    /// Whether to run in dry-run mode, where notifications are not actually sent,
    /// but the program still goes through the motions of monitoring and processing
    /// peer status changes.
    ///
    /// Used for testing and debugging purposes.
    pub dry_run: bool,

    /// Whether to print additional information about the program's operation,
    /// which can be useful for debugging and understanding the program's
    /// behavior, but is entirely optional for everyday use.
    pub verbose: bool,

    /// Whether to print even more detailed information for debugging purposes,
    /// which can be useful for in-depth debugging and understanding of the program's
    /// internal workings, but is generally not needed for regular use.
    ///
    /// This is more verbose than `verbose` is.
    pub debug: bool,
}

impl Settings {
    /// Inherits the configuration directory from the provided `config_dir`
    /// parameter or resolves it from the environment if not provided, updating
    /// the `paths` field of the `Settings` instance accordingly.
    ///
    /// # Parameters
    /// - `config_dir`: An optional string containing the path to the
    ///   configuration directory. If provided, this will be used as the
    ///   configuration directory. If not provided, the method will attempt to
    ///   resolve the default configuration directory from environment variables.
    ///
    /// # Returns
    /// - `Ok(())` if the configuration directory was successfully deduced.
    /// - `Err(String)` if there was an error resolving the default configuration
    ///   directory from environment variables, with a descriptive error message.
    pub fn inherit_config_dir(&mut self, config_dir: &Option<String>) -> Result<(), String> {
        if let Some(dir) = config_dir {
            self.paths.config_dir = path::PathBuf::from(dir);
            return Ok(());
        }

        match file_config::resolve_default_config_directory_from_env() {
            Ok(path) => {
                self.paths.config_dir = path;
                Ok(())
            }
            Err(()) => Err(
                "could not resolve default configuration directory from environment variables"
                    .to_string(),
            ),
        }
    }

    /// Trims whitespace and empty elements from the settings vectors of the backends.
    ///
    /// This is done before the backends are created.
    pub fn clean_up(&mut self) {
        self.slack.trim_urls();
        self.batsign.trim_urls();
        self.command.trim_commands();
    }

    /// Performs a sanity check on the settings, validating that the nested
    /// settings for the monitor, Slack, Batsign, and Command backends are valid,
    /// and that at least one notifier backend is enabled.
    ///
    /// If any issues are found, a vector of descriptive error messages is returned.
    ///
    /// # Returns
    /// - `Ok(())` if the settings are valid.
    /// - `Err(Vec<String>)` if there are issues with the settings, containing a
    ///   vector of descriptive error messages for each issue found.
    pub fn sanity_check(&self) -> Result<(), Vec<String>> {
        let mut vec = Vec::new();

        self.monitor.sanity_check(&mut vec);
        self.slack.sanity_check(&mut vec);
        self.batsign.sanity_check(&mut vec);
        self.command.sanity_check(&mut vec);

        if !self.slack.enabled && !self.batsign.enabled && !self.command.enabled {
            vec.push("At least one notifier backend must be enabled.".to_string());
        }

        if vec.is_empty() { Ok(()) } else { Err(vec) }
    }

    /// Prints the settings in a human-readable format.
    pub fn print(&self) {
        println!("{:#?}", self);
    }

    /// Resolves the paths to the configuration file and peer list file based on
    /// the configuration directory, updating the `paths` field of the `Settings`
    /// instance accordingly.
    ///
    /// # Notes
    /// This should be called after the configuration directory has been resolved
    /// to ensure that the paths to the configuration file and peer list file
    /// are also correct.
    pub fn resolve_resource_paths(&mut self) {
        self.paths.config_file = self.paths.config_dir.join(defaults::CONFIG_FILENAME);
        self.paths.peer_list = self.paths.config_dir.join(defaults::PEER_LIST_FILENAME);
    }

    /// Resolves the path to the `wg` executable and updates the `paths` field of
    /// the `Settings` instance accordingly.
    ///
    /// This merely leverages `wireguard::resolve_wg()`.
    pub fn resolve_wg(&mut self) {
        self.paths.wg = wireguard::resolve_wg();
    }

    /// Applies settings from the provided `file_config::FileConfig` to the
    /// current `Settings` instance, updating the monitor settings, Slack
    /// settings, Batsign settings, and Command settings based on the values
    /// provided in the file configuration.
    ///
    /// # Parameters
    /// - `file_config`: An optional reference to a `file_config::FileConfig`
    ///   containing the settings to apply to the current `Settings` instance.
    ///   If `None`, this method will simply return without making any changes
    ///   to the settings.
    pub fn apply_file(&mut self, file_config: &Option<file_config::FileConfig>) {
        let Some(file_config) = file_config else {
            return;
        };

        self.monitor.apply_file(&file_config.monitor);
        self.slack.apply_file(&file_config.slack);
        self.batsign.apply_file(&file_config.batsign);
        self.command.apply_file(&file_config.command);
    }

    /// Applies settings from the provided `cli::Cli` struct to the current
    /// `Settings` instance, applying values provided in the CLI arguments.
    ///
    /// # Parameters
    /// - `cli`: A reference to a `cli::Cli` struct containing the CLI
    ///   arguments to apply to the current `Settings` instance.
    pub fn apply_cli(&mut self, cli: &cli::Cli) {
        // Config directory is applied separately in `inherit_config_dir`
        // because it affects how other settings are loaded from disk.
        self.resume = cli.resume;
        self.skip_first = cli.skip_first;
        self.disable_timestamps = cli.disable_timestamps;
        self.dry_run = cli.dry_run;
        self.verbose = cli.verbose;
        self.debug = cli.debug;
    }
}
