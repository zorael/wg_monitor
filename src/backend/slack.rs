//! Defines the Slack backend for sending notifications to a Slack channel.
//!
//! The `SlackBackend` composes messages based on the notification context
//! and delta, and sends them to the specified Slack webhook URL.
//!
//! Messages are formatted as JSON payloads according to Slack's requirements,
//! and the backend handles both initial notifications and reminder notifications.

use crate::notify;
use crate::settings;

/// Defines the Slack backend for sending notifications to a Slack channel via webhooks.
pub struct SlackBackend {
    /// Unique identifier for the Slack backend instance, used for logging and
    /// identification purposes.
    #[allow(dead_code)]
    id: usize,

    /// HTTP agent used to send requests to the Slack webhook URL.
    agent: ureq::Agent,

    /// Slack webhook URL to which the notification will be sent. This URL is
    /// provided by Slack when setting up an incoming webhook integration,
    /// and it includes a token for authentication.
    url: String,

    /// Message strings for Slack notifications.
    strings: settings::MessageStrings,

    /// Message strings for Slack reminder notifications.
    reminder_strings: settings::ReminderStrings,

    /// Cached name of the backend instance, which can be used to avoid recomputing
    /// the name on every call to `name()`.
    cached_name: String,
}

impl SlackBackend {
    /// Creates a new instance of `SlackBackend`.
    ///
    /// `cached_name` is computed based on the `id` and is in the format "slack#{id}".
    ///
    /// # Parameters
    /// - `id`: Unique numeric identifier for this backend instance, used
    ///   for logging.
    /// - `agent`: HTTP agent used to send requests to the Slack webhook URL.
    /// - `url`: Slack webhook URL to which the notification will be sent.
    /// - `strings`: Message strings for Slack notifications.
    /// - `reminder_strings`: Message strings for Slack reminder notifications.
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
    /// Returns the unique identifier of this backend instance.
    ///
    /// # Returns
    /// A numeric identifier that uniquely identifies this backend instance.
    #[allow(dead_code)]
    fn id(&self) -> usize {
        self.id
    }

    /// Returns the name of this backend instance, which is in the format "slack#{id}".
    /// The name is used for logging and identification purposes.
    ///
    /// # Returns
    /// A string slice representing the name of this backend instance.
    fn name(&self) -> &str {
        &self.cached_name
    }

    /// Composes a message to be sent to a Slack channel based on the notification
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
        let header_closure = |h: &str| h.to_string();
        notify::prepare_message_body(ctx, delta, &self.strings, header_closure)
            .map(|message| serde_json::json!({ "text": message }).to_string())
    }

    /// Composes a reminder message to be sent to a Slack channel based on the
    /// notification context.
    ///
    /// # Parameters
    /// - `ctx`: The notification context.
    ///
    /// # Returns
    /// - `Some(message)` if a message to send was composed.
    /// - `None` if an empty message was composed, typically meaning no message
    ///   should be sent.
    fn compose_reminder(&self, ctx: &notify::Context) -> Option<String> {
        let header_closure = |h: &str| h.to_string();
        notify::prepare_reminder_body(ctx, &self.reminder_strings, header_closure)
            .map(|message| serde_json::json!({ "text": message }).to_string())
    }

    /// Sends a composed message to a Slack channel by making an HTTP POST
    /// request to the webhook URL with a JSON payload containing the message.
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
        let json: serde_json::Value = serde_json::from_str(message).expect("internal slack json");

        match self.agent.post(&self.url).send_json(json) {
            Ok(mut r) => match r.body_mut().read_to_string() {
                Ok(_) => Ok(None),
                Err(e) => Err(e.to_string()),
            },
            Err(e) => Err(e.to_string()),
        }
    }
}
