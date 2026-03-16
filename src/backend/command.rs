//! A simple external command backend.

use std::collections;
use std::process;

use crate::notify;
use crate::peer;
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
    #[allow(dead_code)]
    fn id(&self) -> usize {
        self.id
    }

    /// Returns the name of this instance of the backend.
    fn name(&self) -> &str {
        &self.cached_name
    }

    /// Builds the message to be sent based on the notification context and the
    /// delta expressing the changes since the last notification.
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
            // Nothing to send, but send the first run header to alert that
            // power is back.
            let message = message
                .replace("\\\\", "\\")
                .replace("\\n", "\n")
                .trim_end()
                .to_string();
            return Some(message);
        }

        message.push_str(body);

        let message = message
            .trim_end()
            .replace("\\\\", "\\")
            .replace("\\n", "\n")
            .to_string();

        Some(message)
    }

    /// Builds the reminder message to be sent based on the notification context.
    fn compose_reminder(&self, ctx: &notify::Context) -> Option<String> {
        let mut message = String::new();
        let body = &notify::format_generic_reminder(ctx, &self.reminder_strings);

        if body.is_empty() {
            return None;
        }

        if !self.reminder_strings.header.is_empty() {
            message.push_str(&self.reminder_strings.header);
            message.push('\n');
        }

        message.push_str(body);

        let message = message
            .trim_end()
            .replace("\\\\", "\\")
            .replace("\\n", "\n")
            .to_string();

        Some(message)
    }

    /// Delivers the already-built message using the external command.
    ///
    /// The command is invoked with the following arguments:
    ///
    /// 1. The composed message to be sent
    /// 2. The path to the peer list file
    /// 3. The number of times the notification loop has run (starting at 0)
    /// 4. A comma-separated string of late keys in the format "key:timestamp"
    /// 5. A comma-separated string of missing keys in the format "key:timestamp"
    /// 6. A comma-separated string of previous late keys in the format "key:timestamp"
    /// 7. A comma-separated string of previous missing keys in the format "key:timestamp"
    /// 8. If a delta is provided, a comma-separated string of keys that became late in the format "key:timestamp"
    /// 9. If a delta is provided, a comma-separated string of keys that went missing in the format "key:timestamp"
    /// 10. If a delta is provided, a comma-separated string of keys that are no longer late in the format "key:timestamp"
    /// 11. If a delta is provided, a comma-separated string of keys that returned in the format "key:timestamp"
    ///
    /// Any parameter for which there is no value (as in, no late keys), the
    /// argument passed but is simply empty.
    fn emit(
        &mut self,
        ctx: &notify::Context,
        delta: Option<&notify::Delta>,
        message: &str,
    ) -> Result<Option<String>, String> {
        let late_keys = format_key_timestamp_pairs(&ctx.peers, &ctx.late_keys);
        let missing_keys = format_key_timestamp_pairs(&ctx.peers, &ctx.missing_keys);
        let previous_late_keys = format_key_timestamp_pairs(&ctx.peers, &ctx.previous_late_keys);
        let previous_missing_keys =
            format_key_timestamp_pairs(&ctx.peers, &ctx.previous_missing_keys);
        let loop_iteration = ctx.loop_iteration.to_string();

        let output = match delta {
            Some(d) => {
                let became_late_keys = format_key_timestamp_pairs(&ctx.peers, &d.became_late_keys);
                let went_missing_keys =
                    format_key_timestamp_pairs(&ctx.peers, &d.went_missing_keys);
                let no_longer_late_keys =
                    format_key_timestamp_pairs(&ctx.peers, &d.no_longer_late_keys);
                let returned_keys = format_key_timestamp_pairs(&ctx.peers, &d.returned_keys);

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

/// Formats the given keys and their corresponding timestamps from the peers map
/// into a comma-separated string in the format "key:timestamp".
/// There must exist a peer in the peers map for each key in the keys slice,
/// or this function will panic.
///
/// Replace the map with the following to avoid panicking if a key is not found.
///
/// ```rust
/// .filter_map(|key| {
///     peers
///         .get(key)
///         .map(|peer| format!("{key}:{}", peer.last_seen_unix))
/// })
/// ```
fn format_key_timestamp_pairs(
    peers: &collections::HashMap<String, peer::WireguardPeer>,
    keys: &[String],
) -> String {
    keys.iter()
        .map(|key| format!("{key}:{}", peers[key].last_seen_unix))
        .collect::<Vec<_>>()
        .join(",")
}

mod test {
    #[allow(unused_imports)]
    use super::*;

    /// Tests the `format_key_timestamp_pairs` function to ensure it correctly
    /// formats the key-timestamp pairs as a comma-separated string.
    #[test]
    fn test_format_key_timestamp_pairs() {
        let mut peers = collections::HashMap::new();

        peers.insert(
            "key1".to_string(),
            peer::WireguardPeer {
                public_key: "key1".to_string(),
                human_name: "Peer 1".to_string(),
                last_seen: None,
                last_seen_unix: 1234567890,
            },
        );

        peers.insert(
            "key2".to_string(),
            peer::WireguardPeer {
                public_key: "key2".to_string(),
                human_name: "Peer 2".to_string(),
                last_seen: None,
                last_seen_unix: 9876543210,
            },
        );

        let keys = vec!["key1".to_string(), "key2".to_string()];
        let result = format_key_timestamp_pairs(&peers, &keys);
        assert_eq!(result, "key1:1234567890,key2:9876543210");

        let keys: Vec<String> = Vec::new();
        let result = format_key_timestamp_pairs(&peers, &keys);
        assert_eq!(result, "");
    }
}
