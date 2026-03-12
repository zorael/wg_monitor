//! A simple external command backend.

use std::process;

use crate::notify;
use crate::settings;

/// The Command backend, which executes an external command to send notifications.
pub struct CommandBackend {
    /// Unique identifier for the Command backend instance, used for
    /// logging and identification purposes.
    id: usize,

    /// The command to execute for notifications.
    command: String,

    /// Message strings for Command notifications.
    strings: settings::MessageStrings,

    /// Message strings for Command reminder notifications.
    reminder_strings: settings::ReminderStrings,

    /// Cached name of the backend instance, which can be used to avoid
    /// recomputing the name on every call to `name()`.
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
    fn compose_message(&self, ctx: &notify::Context, delta: &notify::Delta) -> String {
        let mut message = String::new();
        let body = &notify::format_generic_message(ctx, delta, &self.strings);

        if body.is_empty() {
            return message;
        }

        let header = match ctx.is_first_run() {
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
    fn compose_reminder(&self, ctx: &notify::Context) -> String {
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
    ///
    /// The command is invoked with the following arguments:
    ///
    /// 1. The composed message to be sent
    /// 2. The path to the peer list file
    /// 3. The number of times the notification loop has run (starting at 0)
    /// 4. A comma-separated string of late keys
    /// 5. A comma-separated string of missing keys
    /// 6. A comma-separated string of previous late keys
    /// 7. A comma-separated string of previous missing keys
    /// 8. If a delta is provided, a comma-separated string of keys that became late
    /// 9. If a delta is provided, a comma-separated string of keys that went missing
    /// 10. If a delta is provided, a comma-separated string of keys that are no longer late
    /// 11. If a delta is provided, a comma-separated string of keys that returned
    ///
    /// Any parameter for which there is no value (as in, no late keys), the
    /// argument passed but is simply empty.
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
        let loop_iteration = ctx.loop_iteration.to_string();

        let output = match delta {
            Some(d) => {
                let became_late_keys = d.became_late_keys.join(",");
                let went_missing_keys = d.went_missing_keys.join(",");
                let no_longer_late_keys = d.no_longer_late_keys.join(",");
                let returned_keys = d.returned_keys.join(",");

                process::Command::new(&self.command)
                    .arg(message)
                    .arg(&ctx.peer_list_file_path)
                    .arg(loop_iteration)
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
                .arg(loop_iteration)
                .arg(late_keys)
                .arg(missing_keys)
                .arg(previous_late_keys)
                .arg(previous_missing_keys)
                .output()
                .map_err(|e| e.to_string())?,
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if stdout.is_empty() {
            Ok(None)
        } else {
            Ok(Some(stdout))
        }
    }
}
