//! Defines the `BatsignBackend`, which is a notification backend that sends
//! notifications to the free Batsign service.
//!
//! The `BatsignBackend` implements the `Backend` trait, which requires it to
//! provide methods for composing messages and reminders based on the notification
//! context and delta, as well as a method for emitting the notifications by making
//! HTTP POST requests to the Batsign service.

use crate::notify;
use crate::settings;

/// Defines the Batsign backend for sending notifications to the Batsign service.
///
/// Batsign is a free service that allows you to send notifications to your email
/// address by making HTTP POST requests to a unique URL.
pub struct BatsignBackend {
    /// Unique identifier for the Batsign backend instance, used for
    /// logging and identification purposes.
    id: usize,

    /// HTTP agent used to send requests to the Batsign service.
    agent: ureq::Agent,

    /// Batsign URL to which the notification will be sent.
    ///
    /// This URL is unique to the target email address and includes a token for authentication.
    url: String,

    /// Whether to print the responses to the HTTP requests to the terminal.
    show_response: bool,

    /// Message strings for Batsign alert notifications.
    alert_strings: settings::AlertStrings,

    /// Message strings for Batsign reminder notifications.
    reminder_strings: settings::ReminderStrings,

    /// Cached name of the backend instance, which can be used to avoid recomputing
    /// the name on every call to `name()`.
    ///
    /// The name is in the format "`batsign#{id}:{email}`", where `{id}` is the
    /// unique numeric identifier of the instance, and `{email}` is extracted
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
    /// - `show_response`: Whether to print the responses to the HTTP requests to the terminal.
    /// - `alert_strings`: Message strings for Batsign alert notifications.
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
        show_response: bool,
        alert_strings: &settings::AlertStrings,
        reminder_strings: &settings::ReminderStrings,
    ) -> Self {
        let cached_name = format!(
            "batsign#{id}:{}",
            get_email_from_batsign_url(url).unwrap_or("(?)")
        );

        Self {
            id,
            agent,
            url: url.to_string(),
            show_response,
            alert_strings: alert_strings.clone(),
            reminder_strings: reminder_strings.clone(),
            cached_name,
        }
    }
}

impl super::Backend for BatsignBackend {
    /// Returns the unique identifier of the backend instance.
    fn id(&self) -> usize {
        self.id
    }

    /// Returns the name of this backend instance.
    ///
    /// It is in the format
    /// "`batsign#{id}:{email}`", where `{id}` is the unique numeric identifier
    /// of the instance, and `{email}` is extracted from the Batsign URL.
    fn name(&self) -> &str {
        &self.cached_name
    }

    /// Composes an alert message to be sent via Batsign based on the notification
    /// context and key delta.
    ///
    /// # Parameters
    /// - `ctx`: The notification context.
    /// - `delta`: The changes detected since the last check.
    ///
    /// # Returns
    /// - `Some(String)` if a message to send was composed.
    /// - `None` if the composed message was empty, in which case nothing
    ///   will be sent.
    fn compose_alert(&self, ctx: &notify::Context, delta: &notify::KeyDelta) -> Option<String> {
        let header_closure = |h: &str| format!("Subject: {h}");
        notify::prepare_alert_body(ctx, delta, &self.alert_strings, header_closure)
    }

    /// Composes a reminder message to be sent via Batsign based on the notification
    /// context.
    ///
    /// # Parameters
    /// - `ctx`: The notification context.
    ///
    /// # Returns
    /// - `Some(String)` if a message to send was composed.
    /// - `None` if the composed message was empty, in which case nothing
    ///   will be sent.
    fn compose_reminder(&self, ctx: &notify::Context) -> Option<String> {
        let header_closure = |h: &str| format!("Subject: {h}");
        notify::prepare_reminder_body(ctx, &self.reminder_strings, header_closure)
    }

    /// Sends a composed message to the Batsign service by making an HTTP POST
    /// request to the Batsign URL.
    ///
    /// This implementation ignores `ctx` and `delta` and sends `message` as the
    /// request payload.
    ///
    /// # Parameters
    /// - `ctx`: The notification context (not used in this implementation).
    /// - `delta`: The changes detected since the last alert
    ///   (not used in this implementation).
    /// - `message`: The already-composed message to send.
    ///
    /// # Returns
    /// - `Ok(Some(String))` if the message was sent successfully and the setting to
    ///   output the response is enabled, containing the response body as a string.
    /// - `Ok(None)` if the message was sent successfully but the setting to output
    ///   the response is disabled.
    /// - `Err(String)` if the send attempt failed, containing an error message.
    fn emit(
        &mut self,
        _ctx: &notify::Context,
        _delta: Option<&notify::KeyDelta>,
        message: &str,
    ) -> Result<Option<String>, String> {
        match self.agent.post(&self.url).send(message) {
            Ok(mut r) => match r.body_mut().read_to_string() {
                Ok(output) => {
                    if self.show_response {
                        Ok(Some(output))
                    } else {
                        Ok(None)
                    }
                }
                Err(e) => Err(e.to_string()),
            },
            Err(e) => Err(e.to_string()),
        }
    }

    /// Performs a sanity check on the backend's configuration, specifically
    /// on the URL.
    ///
    /// # Returns
    /// - `Ok(())` if the sanity check passed without any issues.
    /// - `Err(Vec<String>)` if there were issues found during the sanity check,
    ///   containing a vector of descriptive error messages for each issue found.
    fn sanity_check(&self) -> Result<(), Vec<String>> {
        let mut vec = Vec::new();

        if self.url.trim().is_empty() {
            vec.push("Batsign URL must not be empty".to_string());
        } else if get_email_from_batsign_url(&self.url).is_none() {
            vec.push(
                "Batsign URL must contain a valid email address in the format \
                      https://batsign.me/at/{email}/{token}"
                    .to_string(),
            );
        }

        if vec.is_empty() { Ok(()) } else { Err(vec) }
    }
}

/// Extracts the email address from a Batsign URL, which is in the format
/// "`https://batsign.me/at/{email}/{token}`".
///
/// # Parameters
/// - `url`: The Batsign URL from which to extract the email address.
///
/// # Returns
/// - `Some(&str)` if an email address was successfully extracted from the URL.
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
