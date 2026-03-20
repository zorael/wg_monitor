//! Defines the `CommandBackend`, which executes an external command to send
//! notifications.
//!
//! The command is invoked with the composed message and various contextual
//! information as arguments, allowing for flexible integration with custom
//! notification systems or scripts.
//!
//! The `CommandBackend` implements the `Backend` trait, which specifies the
//! interface for all notification backends, including methods for composing
//! messages and sending notifications.

use std::collections;
use std::process;

use crate::notify;
use crate::peer;
use crate::settings;
use crate::utils;

/// Defines the Command backend for sending notifications by executing
/// an external command.
///
/// Commands may be any executable or script that can be invoked from the
/// command line, and will receive the composed message and various contextual
/// information as arguments.
pub struct CommandBackend {
    /// Unique identifier for the Command backend instance, used for
    /// logging and identification purposes.
    id: usize,

    /// The command to execute when sending a notification.
    ///
    /// This should be the path to the executable or script to run.
    command: String,

    /// Message strings for Command notifications.
    strings: settings::MessageStrings,

    /// Message strings for Command reminder notifications.
    reminder_strings: settings::ReminderStrings,

    /// Cached name of the backend instance, which can be used to avoid
    /// recomputing the name on every call to `name()`.
    ///
    /// The name is in the format "command#{id}:{command}", where {id} is the
    /// unique numeric identifier of the instance, and {command} is the command
    /// to execute.
    cached_name: String,
}

impl CommandBackend {
    /// Creates a new instance of `CommandBackend`.
    ///
    /// # Parameters
    /// - `id`: Unique numeric identifier for this backend instance, used
    ///   for logging.
    /// - `command`: The command to execute when sending a notification.
    ///   This should be the path to the executable or script to run.
    /// - `strings`: Message strings for Command notifications.
    /// - `reminder_strings`: Message strings for Command reminder notifications.
    ///
    /// # Returns
    /// A new instance of `CommandBackend` initialized with the provided parameters.
    /// The `cached_name` field is computed based on the `id` and the
    /// `command`, and is in the format "command#{id}:{command}".
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
    ///
    /// # Returns
    /// A numeric identifier that uniquely identifies this backend instance.
    #[allow(dead_code)]
    fn id(&self) -> usize {
        self.id
    }

    /// Returns the name of this backend instance.
    ///
    /// # Returns
    /// A string slice representing the name of this backend instance.
    /// It is in the format "command#{id}:{command}", where {id} is the unique
    /// numeric identifier of the instance, and {command} is the command to execute.
    fn name(&self) -> &str {
        &self.cached_name
    }

    /// Composes a message to be used as argument when executing the command,
    /// based on the notification context and delta.
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
            let message = utils::unescape(&message).trim_end().to_string();
            return Some(message);
        }

        message.push_str(body);

        let message = utils::unescape(&message).trim_end().to_string();
        Some(message)
    }

    /// Composes a reminder message to be used as argument when executing the
    /// command, based on the notification context.
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
            message.push_str(&self.reminder_strings.header);
            message.push('\n');
        }

        message.push_str(body);

        let message = utils::unescape(&message).trim_end().to_string();
        Some(message)
    }

    /// Sends a composed message by executing the configured command with the
    /// message and various contextual information as arguments.
    ///
    /// The command is invoked with the following arguments:
    ///
    /// 1. The composed message to be sent
    /// 2. The path to the peer list file
    /// 3. The number of times the main loop has run (starting at 0, unless --resume was passed)
    /// 4. A comma-separated string of late keys in the format "key:timestamp"
    /// 5. A comma-separated string of missing keys in the format "key:timestamp"
    /// 6. A comma-separated string of previous late keys in the format "key:timestamp"
    /// 7. A comma-separated string of previous missing keys in the format "key:timestamp"
    /// 8. If a delta is provided, a comma-separated string of keys that became
    ///    late in the format "key:timestamp"
    /// 9. If a delta is provided, a comma-separated string of keys that went
    ///    missing in the format "key:timestamp"
    /// 10. If a delta is provided, a comma-separated string of keys that are
    ///     no longer late in the format "key:timestamp"
    /// 11. If a delta is provided, a comma-separated string of keys that
    ///     returned in the format "key:timestamp"
    ///
    /// Any parameter for which there is no value (as in, no late keys), the
    /// argument passed but is simply empty.
    ///
    /// # Parameters
    /// - `ctx`: The notification context.
    /// - `delta`: The changes detected since the last notification, or `None`
    ///   if this is a reminder rather than a new notification.
    /// - `message`: The composed message to send, which is passed as an
    ///   argument to the command.
    ///
    /// # Returns
    /// - `Ok(None)` if the command executed successfully and produced no output.
    /// - `Ok(Some(output))` if the command executed successfully and produced
    ///   output, which is returned as a string.
    /// - `Err(error)` if the command execution failed, with the error message
    ///   returned as a string.
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

/// Formats a list of keys and their corresponding timestamps into a
/// comma-separated string in the format "key:timestamp".
///
/// There must exist a peer in the peers map for each key in the keys slice,
/// or this function will panic.
///
/// Replace the map with the following to avoid panicking if a key is not found.
///
/// ```rust,ignore
/// .filter_map(|key| {
///     peers
///         .get(key)
///         .map(|peer| format!("{key}:{}", peer.last_seen_unix))
/// })
/// ```
///
/// # Parameters
/// - `peers`: A map of peer keys to their corresponding `WireGuardPeer
///   information, which includes the last seen timestamp for each peer.
/// - `keys`: A slice of keys for which to format the key-timestamp pairs.
///   Each key must exist in the `peers` map, or this function will panic.
///
/// # Returns
/// A comma-separated string of key-timestamp pairs in the format "key:timestamp".
///
/// # Panics
/// If any key in the `keys` slice does not exist in the `peers` map,
/// this function will panic with a message indicating the missing key.
fn format_key_timestamp_pairs(
    peers: &collections::HashMap<String, peer::WireGuardPeer>,
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
            peer::WireGuardPeer {
                public_key: "key1".to_string(),
                human_name: "Peer 1".to_string(),
                last_seen: None,
                last_seen_unix: 1234567890,
            },
        );

        peers.insert(
            "key2".to_string(),
            peer::WireGuardPeer {
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
