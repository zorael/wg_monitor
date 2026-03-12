//! Settings for the Command backend.

use crate::file_config;
use crate::settings;
use crate::utils;

/// Command settings structure. This mirrors the runtime settings struct used
/// by the program for Command notifications.
#[derive(Debug)]
pub struct CommandSettings {
    pub strings: settings::MessageStrings,
    pub reminder_strings: settings::ReminderStrings,
    pub enabled: bool,
    pub commands: Vec<String>,
}

impl Default for CommandSettings {
    /// Default values for the Command settings.
    fn default() -> Self {
        Self {
            strings: settings::MessageStrings::default(),
            reminder_strings: settings::ReminderStrings::default(),
            enabled: true,
            commands: Vec::new(),
        }
    }
}

impl CommandSettings {
    /// Applies Command settings from the config file, overriding the default
    /// settings where values are available.
    pub fn apply_file(&mut self, command_config: &file_config::CommandConfig) {
        self.strings.apply_file(&command_config.strings);
        self.reminder_strings
            .apply_file(&command_config.reminder_strings);

        if let Some(enabled) = command_config.enabled {
            self.enabled = enabled;
        }

        if let Some(commands) = command_config.commands.clone() {
            self.commands = commands;
        }
    }

    /// Trims whitespace from the Command strings and removes any empty ones.
    pub fn trim_commands(&mut self) {
        self.commands = utils::trim_vec_of_strings(&self.commands);
    }

    /// Sanity check the Command settings, appending any errors as strings to
    /// the passed vec.
    pub fn sanity_check(&self, vec: &mut Vec<String>) {
        if !self.enabled {
            return;
        }

        if self.commands.is_empty() {
            vec.push("Command backend is enabled but no commands are configured.".to_string());
        }
    }
}
