//! Batsign backend for sending notifications via the free Batsign service.

use reqwest::blocking;
use std::sync;

use crate::notify;
use crate::settings;

/// Batsign backend for sending notifications via the free Batsign service.
pub struct BatsignBackend {
    /// Unique identifier for the Batsign backend instance, used for
    /// logging and identification purposes.
    id: usize,

    /// HTTP client used to send requests to the Batsign service.
    client: sync::Arc<blocking::Client>,

    /// Batsign URL to which the notification will be sent.
    url: String,

    /// Message strings for Batsign notifications.
    strings: settings::MessageStrings,

    /// Message strings for Batsign reminder notifications.
    reminder_strings: settings::ReminderStrings,

    /// Cached name of the backend instance, which can be used to avoid
    /// recomputing the name on every call to `name()`.
    cached_name: String,
}

impl BatsignBackend {
    /// Creates a new instance of the BatsignBackend.
    pub fn new(
        id: usize,
        client: sync::Arc<blocking::Client>,
        url: &str,
        strings: &settings::MessageStrings,
        reminder_strings: &settings::ReminderStrings,
    ) -> Self {
        let cached_name = format!(
            "batsign#{}:{}",
            id,
            get_email_from_batsign_url(url).unwrap_or("(?)")
        );

        Self {
            id,
            client,
            url: url.to_string(),
            strings: strings.clone(),
            reminder_strings: reminder_strings.clone(),
            cached_name,
        }
    }
}

impl super::Backend for BatsignBackend {
    /// Returns the unique identifier of the backend instance.
    #[allow(dead_code)]
    fn id(&self) -> usize {
        self.id
    }

    /// Returns the name of this backend instance. It is in the format
    /// "batsign#{id}:{email}", where {id} is the unique numeric identifier of
    /// the instance, and {email} is extracted from the Batsign URL.
    fn name(&self) -> &str {
        &self.cached_name
    }

    /// Builds the message to be sent to Batsign based on the notification context and delta.
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
            message.push_str(&format!("Subject: {}\n", header));
        }

        message.push_str(body);
        message
            .trim_end()
            .replace("\\\\", "\\")
            .replace("\\n", "\n")
            .to_string()
    }

    /// Builds the reminder message to be sent to Batsign based on the notification context.
    fn compose_reminder(&self, ctx: &notify::Context) -> String {
        let mut message = String::new();
        let body = &notify::format_generic_reminder(ctx, &self.reminder_strings);

        if body.is_empty() {
            return message;
        }

        if !self.reminder_strings.header.is_empty() {
            message.push_str(&format!("Subject: {}\n", &self.reminder_strings.header));
        }

        message.push_str(body);
        message
            .trim_end()
            .replace("\\\\", "\\")
            .replace("\\n", "\n")
            .to_string()
    }

    /// Sends a notification via the Batsign backend by making a POST request
    /// to the specified URL, with the passed message as the request body.
    fn emit(
        &mut self,
        _ctx: &notify::Context,
        _delta: Option<&notify::Delta>,
        message: &str,
    ) -> Result<Option<String>, String> {
        match self.client.post(&self.url).body(message.to_string()).send() {
            Ok(resp) if resp.status().is_success() => Ok(None),
            Ok(resp) => Err(format!("HTTP {}", resp.status())),
            Err(e) => Err(e.to_string()),
        }
    }
}

/// Extracts an email address from a single Batsign URL, returning it as a `&str`.
fn get_email_from_batsign_url(url: &str) -> Option<&str> {
    // https://batsign.me/at/{email}/{token}
    //       ^^          ^  ^       ^       ^?
    let mut parts = url.split('/');

    while let Some(p) = parts.next() {
        if p == "at" {
            let email = parts.next()?;
            return email.contains('@').then_some(email);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    /// Tests for the `get_email_from_batsign_url` function, which extracts
    /// an email address from a Batsign URL.
    #[test]
    fn test_get_email_from_batsign_url() {
        let url = "https://batsign.me/at/test@example.com/token";
        let email = super::get_email_from_batsign_url(url);
        assert_eq!(email, Some("test@example.com"));

        let url = "https://batsign.me/at/example@test.com/token";
        let email = super::get_email_from_batsign_url(url);
        assert_eq!(email, Some("example@test.com"));

        let url = "https://batsign.me/at/blork/token";
        let email = super::get_email_from_batsign_url(url);
        assert_eq!(email, None);

        let url = "https://batsign.me/";
        let email = super::get_email_from_batsign_url(url);
        assert_eq!(email, None);

        let url = "";
        let email = super::get_email_from_batsign_url(url);
        assert_eq!(email, None);
    }
}
