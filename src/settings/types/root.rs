//! Defines the main `Settings` struct that holds all runtime settings for the
//! application, including monitoring settings, notifier settings, and resource paths.

use std::path;

use crate::cli;
use crate::defaults;
use crate::file_config;

/// Application runtime settings root struct.
#[derive(Debug, Default)]
pub struct Settings {
    /// Monitor settings for the WireGuard interface and connection status.
    pub monitor: super::MonitorSettings,

    /// Slack settings.
    pub slack: super::SlackSettings,

    /// Batsign settings.
    pub batsign: super::BatsignSettings,

    /// Command settings.
    pub command: super::CommandSettings,

    /// Paths to resources, resolved at runtime.
    pub paths: super::PathBufs,

    /// Whether to skip sending notifications specifically about program startup.
    pub resume: bool,

    /// Whether to skip the first run and thus the first notification.
    pub skip_first: bool,

    /// Whether to include `[HH:MM:SS]` timestamps in terminal output.
    /// This is disabled by default when the program detects that it's not running in a terminal.
    pub disable_timestamps: bool,

    /// Whether to send notifications or just echo what would be done.
    pub dry_run: bool,

    /// Whether to print additional verbose information.
    pub verbose: bool,

    /// Whether to print additional debug information.
    pub debug: bool,
}

impl Settings {
    /// Applies the configuration directory setting, resolving the resource paths
    /// based on the provided directory or the default.
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

    /// Cleans up settings by trimming whitespace from URLs and removing empty URLs.
    pub fn clean_up(&mut self) {
        self.slack.trim_urls();
        self.batsign.trim_urls();
        self.command.trim_commands();
    }

    /// Sanity check settings, returning a list of strings of errors in the
    /// `Err` case if any are found.
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

    /// Pretty-prints all runtime settings in a humanly-readable format.
    pub fn print(&self) {
        println!("{:#?}", self);

        if self.dry_run {
            println!();
            println!("[!] DRY RUN");
        }
    }

    /// Resolves the resource paths based on the config directory.
    pub fn resolve_resource_paths(&mut self) {
        self.paths.config_file = self.paths.config_dir.join(defaults::CONFIG_FILENAME);
        self.paths.peer_list = self.paths.config_dir.join(defaults::PEER_LIST_FILENAME);
    }

    /// Applies config file settings to the default settings, returning the resulting settings.
    pub fn apply_file(&mut self, file_config: &Option<file_config::FileConfig>) {
        let Some(file_config) = file_config else {
            return;
        };

        self.monitor.apply_file(&file_config.monitor);
        self.slack.apply_file(&file_config.slack);
        self.batsign.apply_file(&file_config.batsign);
        self.command.apply_file(&file_config.command);
    }

    /// Applies CLI settings, returning the resulting settings.
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
