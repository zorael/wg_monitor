//! Defines the `BatsignBackend`, which is a notification backend that sends
//! notifications to the free Batsign service.
//!
//! The `BatsignBackend` implements the `Backend` trait, which requires it to
//! provide methods for composing messages and reminders based on the notification
//! context and delta, as well as a method for emitting the notifications by making
//! HTTP POST requests to the Batsign service.

use crate::notify;
use crate::settings;
use crate::utils;

/// Defines the Batsign backend for sending notifications to the Batsign service.
///
/// Batsign is a free service that allows you to send notifications to your email
/// address by making HTTP POST requests to a unique URL. The `BatsignBackend`
/// composes messages based on the notification context and delta, and sends them
/// to the specified Batsign URL.
pub struct BatsignBackend {
    /// Unique identifier for the Batsign backend instance, used for
    /// logging and identification purposes.
    id: usize,

    /// HTTP agent used to send requests to the Batsign service.
    agent: ureq::Agent,

    /// Batsign URL to which the notification will be sent. This URL is unique
    /// to the target email address and includes a token for authentication.
    url: String,

    /// Message strings for Batsign notifications.
    strings: settings::MessageStrings,

    /// Message strings for Batsign reminder notifications.
    reminder_strings: settings::ReminderStrings,

    /// Cached name of the backend instance, which can be used to avoid recomputing
    /// the name on every call to `name()`.
    ///
    /// The name is in the format "batsign#{id}:{email}", where {id} is the
    /// unique numeric identifier of the instance, and {email} is extracted
    /// from the Batsign URL.
    cached_name: String,
}

impl BatsignBackend {
    /// Creates a new instance of `BatsignBackend`.
    ///
    /// # Parameters
    /// - `id`: Unique numeric identifier for this backend instance, used
    ///   for logging.
    /// - `agent`: HTTP agent used to send requests to the Batsign service.
    /// - `url`: Batsign URL to which the notification will be sent.
    /// - `strings`: Message strings for Batsign notifications.
    /// - `reminder_strings`: Message strings for Batsign reminder notifications.
    ///
    /// # Returns
    /// A new instance of `BatsignBackend` initialized with the provided parameters.
    /// The `cached_name` field is computed based on the `id` and the email
    /// extracted from the `url`.
    pub fn new(
        id: usize,
        agent: ureq::Agent,
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
            agent,
            url: url.to_string(),
            strings: strings.clone(),
            reminder_strings: reminder_strings.clone(),
            cached_name,
        }
    }
}

impl super::Backend for BatsignBackend {
    /// Returns the unique identifier of the backend instance.
    ///
    /// # Returns
    /// A numeric identifier that uniquely identifies this backend instance.
    #[allow(dead_code)]
    fn id(&self) -> usize {
        self.id
    }

    /// Returns the name of this backend instance. It is in the format
    /// "batsign#{id}:{email}", where {id} is the unique numeric identifier
    /// of the instance, and {email} is extracted from the Batsign URL.
    ///
    /// # Returns
    /// A string slice representing the name of this backend instance.
    fn name(&self) -> &str {
        &self.cached_name
    }

    /// Composes a message to be sent to Batsign based on the notification
    /// context and delta.
    ///
    /// # Parameters
    /// - `ctx`: The notification context.
    /// - `delta`: The changes detected since the last notification.
    ///
    /// # Returns
    /// - `Some(message)` if a message to send was composed.
    /// - `None` if an empty message was composed, typically meaning no message
    ///   should be sent.
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
            message.push_str(&format!("Subject: {}\n", header));
        }

        if body.is_empty() && ctx.is_first_run() {
            if header.is_empty() {
                // Nothing to send on first run and no header,
                // so just skip sending a message.
                return None;
            }

            // Nothing to send, but send the first run header to alert that
            // power is back.
            let message = utils::unescape(&message).trim_end().to_string();
            return Some(message);
        }

        message.push_str(body);

        let message = utils::unescape(&message).trim_end().to_string();
        Some(message)
    }

    /// Composes a reminder message to be sent to Batsign based on the notification
    /// context.
    ///
    /// # Parameters
    /// - `ctx`: The notification context.
    ///
    /// # Returns
    /// - `Some(message)` if a message to send was composed.
    /// - `None` if an empty message was composed, typically meaning no message
    ///   should be sent.
    fn compose_reminder(&self, ctx: &notify::Context) -> Option<String> {
        let mut message = String::new();
        let body = &notify::format_generic_reminder(ctx, &self.reminder_strings);

        if body.is_empty() {
            return None;
        }

        if !self.reminder_strings.header.is_empty() {
            message.push_str(&format!("Subject: {}\n", &self.reminder_strings.header));
        }

        message.push_str(body);

        let message = utils::unescape(&message).trim_end().to_string();
        Some(message)
    }

    /// Sends a composed message to the Batsign service by making an HTTP POST
    /// request to the Batsign URL.
    ///
    /// This implementation ignores `ctx` and `delta` and sends `message` as the
    /// request payload.
    ///
    /// # Parameters
    /// - `ctx`: The notification context (not used in this implementation).
    /// - `delta`: The changes detected since the last notification
    ///   (not used in this implementation).
    /// - `message`: The already-composed message to send.
    ///
    /// # Returns
    /// - `Ok(None)` if the message was sent successfully.
    /// - `Err(error)` if the send attempt failed.
    fn emit(
        &mut self,
        _ctx: &notify::Context,
        _delta: Option<&notify::Delta>,
        message: &str,
    ) -> Result<Option<String>, String> {
        match self.agent.post(&self.url).send(message) {
            Ok(mut r) => match r.body_mut().read_to_string() {
                Ok(_) => Ok(None),
                Err(e) => Err(e.to_string()),
            },
            Err(e) => Err(e.to_string()),
        }
    }
}

/// Extracts the email address from a Batsign URL, which is in the format
/// "https://batsign.me/at/{email}/{token}".
///
/// # Parameters
/// - `url`: The Batsign URL from which to extract the email address.
///
/// # Returns
/// - `Some(email)` if an email address was successfully extracted from the URL.
/// - `None` if the URL does not contain a valid email address in the
///   expected format.
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
    /// the email address from a Batsign URL.
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
