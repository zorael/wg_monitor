//! Batsign backend for sending notifications via the free Batsign service.

use reqwest::blocking;
use std::sync;

use crate::backend;
use crate::notify;
use crate::settings;

/// Batsign backend for sending notifications via the free Batsign service.
pub struct BatsignBackend {
    /// Unique identifier for the Batsign backend instance, used for logging and identification purposes.
    id: usize,

    /// HTTP client used to send requests to the Batsign service.
    client: sync::Arc<blocking::Client>,

    /// Batsign URL to which the notification will be sent.
    url: String,

    /// Message strings for Batsign notifications.
    strings: settings::MessageStrings,

    /// Message strings for Batsign reminder notifications.S
    reminder_strings: settings::ReminderStrings,
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
        Self {
            id,
            client,
            url: url.to_owned(),
            strings: strings.clone(),
            reminder_strings: reminder_strings.clone(),
        }
    }
}

impl backend::Backend for BatsignBackend {
    /// Returns the name of the backend, which is "batsign" in this case.
    fn name(&self) -> String {
        // This can be cached if it turns out to be a hotspot.
        format!(
            "batsign#{}:{}",
            self.id,
            get_email_from_batsign_url(&self.url).unwrap_or("(?)")
        )
    }

    /// Builds the message to be sent to Batsign based on the notification context and delta.
    fn build_message(&self, ctx: &notify::Context, delta: &notify::Delta) -> String {
        let mut message = String::new();
        let header = match ctx.first_run {
            true => &self.strings.first_run_header,
            false => &self.strings.header,
        };

        message.push_str(&format!("Subject: {}\n", header));
        message.push_str(&notify::format_generic_message(ctx, delta, &self.strings));
        message
    }

    /// Builds the reminder message to be sent to Batsign based on the notification context.
    fn build_reminder(&self, ctx: &notify::Context) -> String {
        let mut message = String::new();
        message.push_str(&format!("Subject: {}\n", &self.reminder_strings.header));
        message.push_str(&notify::format_generic_reminder(
            ctx,
            &self.reminder_strings,
        ));
        message
    }

    /// Sends a notification via the Batsign backend by making a POST request
    /// to the specified URL with the message as the body.
    fn send(&mut self, message: &str) -> Result<(), String> {
        match self.client.post(&self.url).body(message.to_owned()).send() {
            Ok(resp) if resp.status().is_success() => Ok(()),
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
