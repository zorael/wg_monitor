//! Slack backend for sending notifications to a Slack channel via webhooks.
//!
//! This module defines the `SlackBackend` struct, which implements the `Backend` trait
//! for sending notifications to Slack. It includes methods for building messages and
//! reminders based on the notification context and delta, as well as sending the
//! notifications via HTTP POST requests to the Slack API.

use reqwest::blocking;
use std::sync;

use crate::notify;
use crate::settings;

/// Defines the Slack backend for sending notifications to a Slack channel.
pub struct SlackBackend {
    /// Unique identifier for the Slack backend instance, used for logging and
    /// identification purposes.
    id: usize,

    /// HTTP client used to send requests to the Slack API.
    client: sync::Arc<blocking::Client>,

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
        client: sync::Arc<blocking::Client>,
        url: &str,
        strings: &settings::MessageStrings,
        reminder_strings: &settings::ReminderStrings,
    ) -> Self {
        let cached_name = format!("slack#{}", id);

        Self {
            id,
            client,
            url: url.to_owned(),
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
            message.push_str(&format!("{header}\n"));
        }

        message.push_str(body);
        serde_json::json!({ "text": format!("{message}") }).to_string()
    }

    /// Builds the reminder message to be sent to Slack based on the notification context.
    fn build_reminder(&self, ctx: &notify::Context) -> String {
        let mut message = String::new();
        let body = &notify::format_generic_reminder(ctx, &self.reminder_strings);

        if body.is_empty() {
            return message;
        }

        if !self.reminder_strings.header.is_empty() {
            message.push_str(&format!("{}\n", &self.reminder_strings.header));
        }

        message.push_str(body);
        serde_json::json!({ "text": format!("{message}") }).to_string()
    }

    /// Sends a notification via the Slack backend by making a POST request
    /// to the specified webhook URL, with the passed message string as parsed
    /// into a JSON payload.
    fn emit(
        &mut self,
        _ctx: &notify::Context,
        _delta: Option<&notify::Delta>,
        message: &str,
    ) -> Result<(), String> {
        let json: serde_json::Value = serde_json::from_str(message).expect("internal slack json");

        match self.client.post(&self.url).json(&json).send() {
            Ok(resp) if resp.status().is_success() => Ok(()),
            Ok(resp) => Err(format!("HTTP {}", resp.status())),
            Err(e) => Err(e.to_string()),
        }
    }
}
