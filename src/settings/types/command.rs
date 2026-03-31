//! Settings structures for the Command notification backend, which executes
//! user-defined external commands to send notifications.

use crate::file_config;
use crate::settings;
use crate::utils;

/// Runtime settings for the Command notification backend, including message
/// strings, enabled status, and notification commands.
#[derive(Debug, Default)]
pub struct CommandSettings {
    /// Message strings for Command alert notifications.
    pub alert_strings: settings::AlertStrings,

    /// Message strings for Command reminder notifications.
    pub reminder_strings: settings::ReminderStrings,

    /// Whether Command notifications are enabled.
    pub enabled: bool,

    /// The command strings to execute for notifications.
    ///
    /// Each command string is executed as a separate non-asynchronous process.
    pub commands: Vec<String>,

    /// Whether to print the output of the executed comomands to the terminal.
    pub show_output: bool,
}

impl CommandSettings {
    /// Applies settings from a `file_config::CommandConfig` to the current
    /// `CommandSettings` instance.
    ///
    /// # Parameters
    /// - `command_config`: The `file_config::CommandConfig` containing the
    ///   settings to apply to the current `CommandSettings` instance.
    pub fn apply_file(&mut self, command_config: &file_config::CommandConfig) {
        self.alert_strings.apply_file(&command_config.alert_strings);
        self.reminder_strings
            .apply_file(&command_config.reminder_strings);

        if let Some(enabled) = command_config.enabled {
            self.enabled = enabled;
        }

        if let Some(commands) = &command_config.commands {
            self.commands = commands.clone();
        }

        if let Some(show_output) = command_config.show_output {
            self.show_output = show_output;
        }
    }

    /// Trims whitespace from the Command strings in the settings, which can
    /// help to avoid issues with command strings that have leading or trailing
    /// whitespace that could cause problems when executing the commands.
    pub fn trim_commands(&mut self) {
        self.commands = utils::trim_vec_of_strings(&self.commands);
    }

    /// Performs a sanity check on the Command settings, validating that if
    /// Command notifications are enabled, there are at the same time command
    /// strings configured.
    ///
    /// If any issues are found, descriptive error messages are added to the
    /// provided vector of strings.
    ///
    /// # Parameters
    /// - `vec`: A mutable reference to a vector of strings where error messages
    ///   will be added if any issues are found with the Command settings.
    ///   If the settings are valid, this vector will remain unchanged.
    pub fn sanity_check(&self, vec: &mut Vec<String>) {
        if !self.enabled {
            return;
        }

        if self.commands.is_empty() {
            vec.push("Command backend is enabled but no commands are configured.".to_string());
        }
    }
}
