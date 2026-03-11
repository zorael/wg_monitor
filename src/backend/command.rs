//! FIXME

use crate::notify;
use crate::settings;

pub struct CommandBackend {
    id: usize,
    command: String,
    strings: settings::MessageStrings,
    reminder_strings: settings::ReminderStrings,
    cached_name: String,
}

impl CommandBackend {
    pub fn new(
        id: usize,
        command: &str,
        strings: &settings::MessageStrings,
        reminder_strings: &settings::ReminderStrings,
    ) -> Self {
        let cached_name = format!("command#{}:{}", id, command);

        Self {
            id,
            command: command.to_owned(),
            strings: strings.clone(),
            reminder_strings: reminder_strings.clone(),
            cached_name,
        }
    }
}

impl super::Backend for CommandBackend {
    fn id(&self) -> usize {
        self.id
    }

    fn name(&self) -> &str {
        &self.cached_name
    }

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
        message.trim_end().to_owned()
    }

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
        message.trim_end().to_owned()
    }

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

                std::process::Command::new(&self.command)
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
            None => std::process::Command::new(&self.command)
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
        println!("{stdout}");

        if !output.status.success() {
            return Err(stdout);
        }

        Ok(Some(stdout))
    }
}
