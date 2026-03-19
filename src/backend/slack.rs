//! Slack backend for sending notifications to a Slack channel via webhooks.
//!
//! This module defines the `SlackBackend` struct, which implements the `Backend` trait
//! for sending notifications to Slack. It includes methods for building messages and
//! reminders based on the notification context and delta, as well as sending the
//! notifications via HTTP POST requests to the Slack API.

use crate::notify;
use crate::settings;

/// Defines the Slack backend for sending notifications to a Slack channel.
pub struct SlackBackend {
    /// Unique identifier for the Slack backend instance, used for
    /// logging and identification purposes.
    #[allow(dead_code)]
    id: usize,

    /// HTTP client used to send requests to the Slack API.
    agent: ureq::Agent,

    /// Slack webhook URL to which the notification will be sent.
    url: String,

    /// Message strings for Slack notifications.
    strings: settings::MessageStrings,

    /// Message strings for Slack reminder notifications.
    reminder_strings: settings::ReminderStrings,

    /// Cached name of the backend instance, which can be used to avoid
    /// recomputing the name on every call to `name()`.
    cached_name: String,
}

#[allow(dead_code)]
impl SlackBackend {
    /// Creates a new instance of SlackBackend.
    pub fn new(
        id: usize,
        agent: ureq::Agent,
        url: &str,
        strings: &settings::MessageStrings,
        reminder_strings: &settings::ReminderStrings,
    ) -> Self {
        let cached_name = format!("slack#{}", id);

        Self {
            id,
            agent,
            url: url.to_string(),
            strings: strings.clone(),
            reminder_strings: reminder_strings.clone(),
            cached_name,
        }
    }
}

impl super::Backend for SlackBackend {
    /// Returns the unique identifier of the backend instance.
    #[allow(dead_code)]
    fn id(&self) -> usize {
        self.id
    }

    /// Returns the name of this backend instance. It is in the format
    /// "slack#{id}", where {id} is the unique numeric identifier of instance.
    fn name(&self) -> &str {
        &self.cached_name
    }

    /// Builds the message to be sent to Slack based on the notification context and delta.
    fn compose_message(&self, ctx: &notify::Context, delta: &notify::Delta) -> Option<String> {
        let mut message = String::new();
        let body = &notify::format_generic_message(ctx, delta, &self.strings);

        if body.is_empty() && !ctx.is_first_run() {
            // Nothing to send. If it's the first run, we still want to send the
            // "first run" banner, even if there are no changes.
            return None;
        }

        let header = match ctx.is_first_run() {
            true => &self.strings.first_run_header,
            false => &self.strings.header,
        };

        if !header.is_empty() {
            message.push_str(header);
            message.push('\n');
        }

        if body.is_empty() && ctx.is_first_run() {
            if header.is_empty() {
                // Nothing to send on first run and no header,
                // so just skip sending a message.
                return None;
            }

            // Nothing to send, but send the first run header to alert that
            // power is back.
            let message = message
                .replace("\\\\", "\\")
                .replace("\\n", "\n")
                .trim_end()
                .to_string();
            let json = serde_json::json!({ "text": format!("{message}") }).to_string();
            return Some(json);
        }

        message.push_str(body);

        let message = escape_common_json_characters(&message);
        let message = message
            .replace("\\\\", "\\")
            .replace("\\n", "\n")
            .trim_end()
            .to_string();

        Some(serde_json::json!({ "text": format!("{message}") }).to_string())
    }

    /// Builds the reminder message to be sent to Slack based on the notification context.
    fn compose_reminder(&self, ctx: &notify::Context) -> Option<String> {
        let mut message = String::new();
        let body = &notify::format_generic_reminder(ctx, &self.reminder_strings);

        if body.is_empty() && !ctx.is_first_run() {
            // Nothing to send. If it's the first run, we still want to send the
            // "first run" banner, even if there are no changes.
            return None;
        }

        if !self.reminder_strings.header.is_empty() {
            message.push_str(&self.reminder_strings.header);
            message.push('\n');
        }

        if body.is_empty() && ctx.is_first_run() {
            // Nothing to send, but send the first run header to alert that
            // power is back.
            let message = message
                .replace("\\\\", "\\")
                .replace("\\n", "\n")
                .trim_end()
                .to_string();
            let json = serde_json::json!({ "text": format!("{message}") }).to_string();
            return Some(json);
        }

        message.push_str(body);

        let message = escape_common_json_characters(&message);
        let message = message
            .replace("\\\\", "\\")
            .replace("\\n", "\n")
            .trim_end()
            .to_string();

        Some(serde_json::json!({ "text": format!("{message}") }).to_string())
    }

    /// Sends a notification via the Slack backend by making a POST request
    /// to the specified webhook URL, with the passed message string as parsed
    /// into a JSON payload.
    fn emit(
        &mut self,
        _ctx: &notify::Context,
        _delta: Option<&notify::Delta>,
        message: &str,
    ) -> Result<Option<String>, String> {
        let json: serde_json::Value = serde_json::from_str(message).expect("internal slack json");

        let resp = self.agent.post(&self.url).send_json(json);

        match resp {
            Ok(mut r) => match r.body_mut().read_to_string() {
                Ok(_) => Ok(None),
                Err(e) => Err(e.to_string()),
            },
            Err(e) => Err(e.to_string()),
        }
    }
}

/// Escapes common characters in the input string that may interfere with JSON formatting,
/// such as backslashes, quotes, and curly braces.
fn escape_common_json_characters(input: &str) -> String {
    input
        .replace("\\", "\\\\")
        .replace("\"", "\\\"")
        .replace("{", "\\{")
        .replace("}", "\\}")
}
