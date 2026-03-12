//! A simple external command backend.

use std::process;

use crate::notify;
use crate::settings;

/// The Command backend, which executes an external command to send notifications.
pub struct CommandBackend {
    id: usize,
    command: String,
    strings: settings::MessageStrings,
    reminder_strings: settings::ReminderStrings,
    cached_name: String,
}

impl CommandBackend {
    /// Creates a new instance of CommandBackend.
    pub fn new(
        id: usize,
        command: &str,
        strings: &settings::MessageStrings,
        reminder_strings: &settings::ReminderStrings,
    ) -> Self {
        let cached_name = format!("command#{}:{}", id, command);

        Self {
            id,
            command: command.to_string(),
            strings: strings.clone(),
            reminder_strings: reminder_strings.clone(),
            cached_name,
        }
    }
}

impl super::Backend for CommandBackend {
    /// Returns the unique identifier of the backend instance.
    fn id(&self) -> usize {
        self.id
    }

    /// Returns the name of this instance of the backend.
    fn name(&self) -> &str {
        &self.cached_name
    }

    /// Builds the message to be sent based on the notification context and the
    /// delta expressing the changes since the last notification.
    fn build_message(&self, ctx: &notify::Context, delta: &notify::Delta) -> String {
        let mut message = String::new();
        let body = &notify::format_generic_message(ctx, delta, &self.strings);

        if body.is_empty() {
            return message;
        }

        let header = match ctx.first_run {
            true => &self.strings.first_run_header,
            false => &self.strings.header,
        };

        if !header.is_empty() {
            message.push_str(header);
            message.push('\n');
        }

        message.push_str(body);
        message.trim_end().to_string()
    }

    /// Builds the reminder message to be sent based on the notification context.
    fn build_reminder(&self, ctx: &notify::Context) -> String {
        let mut message = String::new();
        let body = &notify::format_generic_reminder(ctx, &self.reminder_strings);

        if body.is_empty() {
            return message;
        }

        if !self.reminder_strings.header.is_empty() {
            message.push_str(&self.reminder_strings.header);
            message.push('\n');
        }

        message.push_str(body);
        message.trim_end().to_string()
    }

    /// Delivers the already-built message using the external command.
    fn emit(
        &mut self,
        ctx: &notify::Context,
        delta: Option<&notify::Delta>,
        message: &str,
    ) -> Result<Option<String>, String> {
        let late_keys = ctx.late_keys.join(",");
        let missing_keys = ctx.missing_keys.join(",");
        let previous_late_keys = ctx.previous_late_keys.join(",");
        let previous_missing_keys = ctx.previous_missing_keys.join(",");
        let first_run = match ctx.first_run {
            true => "1",
            false => "0",
        };

        let output = match delta {
            Some(d) => {
                let became_late_keys = d.became_late_keys.join(",");
                let went_missing_keys = d.went_missing_keys.join(",");
                let no_longer_late_keys = d.no_longer_late_keys.join(",");
                let returned_keys = d.returned_keys.join(",");

                process::Command::new(&self.command)
                    .arg(message)
                    .arg(&ctx.peer_list_file_path)
                    .arg(first_run)
                    .arg(late_keys)
                    .arg(missing_keys)
                    .arg(previous_late_keys)
                    .arg(previous_missing_keys)
                    .arg(became_late_keys)
                    .arg(went_missing_keys)
                    .arg(no_longer_late_keys)
                    .arg(returned_keys)
                    .output()
                    .map_err(|e| e.to_string())?
            }
            None => process::Command::new(&self.command)
                .arg(message)
                .arg(&ctx.peer_list_file_path)
                .arg(first_run)
                .arg(late_keys)
                .arg(missing_keys)
                .arg(previous_late_keys)
                .arg(previous_missing_keys)
                .output()
                .map_err(|e| e.to_string())?,
        };

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if !output.status.success() {
            return Err(stdout);
        }

        Ok(Some(stdout))
    }
}
