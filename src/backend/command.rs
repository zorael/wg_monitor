//! Defines the `CommandBackend`, which executes an external command to send
//! notifications.
//!
//! The command is invoked with the composed message and various contextual
//! information as arguments, allowing for integration with custom
//! notification systems or scripts.
//!
//! The `CommandBackend` implements the `Backend` trait, which specifies the
//! interface for all notification backends, including methods for composing
//! messages and sending notifications.

use std::collections;
use std::process;

use crate::notify;
use crate::settings;
use crate::wireguard;

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

    /// Whether to print the standard output of the executed commands to the terminal.
    show_output: bool,

    /// Message strings for Command alert notifications.
    alert_strings: settings::AlertStrings,

    /// Message strings for Command reminder notifications.
    reminder_strings: settings::ReminderStrings,

    /// Cached name of the backend instance, which can be used to avoid
    /// recomputing the name on every call to `name()`.
    ///
    /// The name is in the format "`command#{id}:{command}`", where `{id}` is the
    /// unique numeric identifier of the instance, and `{command}` is the command
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
    /// - `show_output`: Whether to print the output of the executed commands to the terminal.
    /// - `alert_strings`: Message strings for Command alert notifications.
    /// - `reminder_strings`: Message strings for Command reminder notifications.
    ///
    /// # Returns
    /// A new instance of `CommandBackend` initialized with the provided parameters.
    /// The `cached_name` field is computed based on the `id` and the
    /// `command`, and is in the format "command#{id}:{command}".
    pub fn new(
        id: usize,
        command: &str,
        show_output: bool,
        alert_strings: &settings::AlertStrings,
        reminder_strings: &settings::ReminderStrings,
    ) -> Self {
        let cached_name = format!("command#{id}:{command}");

        Self {
            id,
            command: command.to_string(),
            show_output,
            alert_strings: alert_strings.clone(),
            reminder_strings: reminder_strings.clone(),
            cached_name,
        }
    }
}

impl super::Backend for CommandBackend {
    /// Returns the unique identifier of the backend instance.
    fn id(&self) -> usize {
        self.id
    }

    /// Returns the name of this backend instance.
    ///
    /// It is in the format "`command#{id}:{command}`", where `{id}` is the unique
    /// numeric identifier of the instance, and `{command}` is the command to execute.
    fn name(&self) -> &str {
        &self.cached_name
    }

    /// Composes an alert message to be used as argument when executing the command,
    /// based on the notification context and key delta.
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
        let header_closure = |h: &str| h.to_string();
        notify::prepare_alert_body(ctx, delta, &self.alert_strings, header_closure)
    }

    /// Composes a reminder message to be used as argument when executing the
    /// command, based on the notification context.
    ///
    /// # Parameters
    /// - `ctx`: The notification context.
    ///
    /// # Returns
    /// - `Some(String)` if a message to send was composed.
    /// - `None` if the composed message was empty, in which case nothing
    ///   will be sent.
    fn compose_reminder(&self, ctx: &notify::Context) -> Option<String> {
        let header_closure = |h: &str| h.to_string();
        notify::prepare_reminder_body(ctx, &self.reminder_strings, header_closure)
    }

    /// Sends a composed message by executing the configured command with the
    /// message and various contextual information as arguments.
    ///
    /// The command is invoked with the following as command-line arguments
    /// in order (as in, argument 1 is `$1`):
    ///
    /// 1. The composed message to be sent
    /// 2. The path to the peer list file
    /// 3. The number of times the main loop has run (starting at 0, unless
    ///    --resume was passed, in which case it starts at 1)
    /// 4. A comma-separated string of lost keys in the format "`key:timestamp`"
    /// 5. A comma-separated string of missing keys in the format "`key:timestamp`"
    /// 6. If a key delta is provided, a comma-separated string of keys that are now
    ///    lost in the format "`key:timestamp`"
    /// 7. If a key delta is provided, a comma-separated string of keys that are now
    ///    missing in the format "`key:timestamp`"
    /// 8. If a key delta is provided, a comma-separated string of keys that were
    ///    lost (but are no longer) in the format "`key:timestamp`"
    /// 9. If a key delta is provided, a comma-separated string of keys that
    ///    were missing (but are no longer) in the format "`key:timestamp`"
    ///
    /// Any parameter for which there is no value (as in, no lost keys,
    /// no missing keys, etc) missing, etc), the argument passed but is simply
    /// an empty string `""`.
    ///
    /// # Parameters
    /// - `ctx`: The notification context.
    /// - `delta`: The changes detected since the last alert.
    /// - `message`: The composed message to send, which is passed as argument
    ///   `$1` to the command.
    ///
    /// # Returns
    /// - `Ok(None)` if the command executed successfully and produced no output.
    /// - `Ok(Some(String))` if the command executed successfully and produced
    ///   output, which is returned as a string.
    /// - `Err(String)` if the command execution failed, with the error message
    ///   returned as a string.
    fn emit(
        &mut self,
        ctx: &notify::Context,
        delta: Option<&notify::KeyDelta>,
        message: &str,
    ) -> Result<Option<String>, String> {
        let lost_keys = format_key_timestamp_pairs(&ctx.peers, &ctx.lost_keys);
        let missing_keys = format_key_timestamp_pairs(&ctx.peers, &ctx.missing_keys);
        let loop_iteration = ctx.loop_iteration.to_string();

        let output = match delta {
            Some(d) => {
                let now_lost_keys = format_key_timestamp_pairs(&ctx.peers, &d.now_lost);
                let was_lost_keys = format_key_timestamp_pairs(&ctx.peers, &d.was_lost);
                let now_missing_keys = format_key_timestamp_pairs(&ctx.peers, &d.now_missing);
                let was_missing_keys = format_key_timestamp_pairs(&ctx.peers, &d.was_missing);

                process::Command::new(&self.command)
                    .arg(message)
                    .arg(ctx.peer_list.display().to_string())
                    .arg(loop_iteration)
                    .arg(lost_keys)
                    .arg(missing_keys)
                    .arg(now_lost_keys)
                    .arg(now_missing_keys)
                    .arg(was_lost_keys)
                    .arg(was_missing_keys)
                    .output()
                    .map_err(|e| e.to_string())?
            }
            None => process::Command::new(&self.command)
                .arg(message)
                .arg(ctx.peer_list.display().to_string())
                .arg(loop_iteration)
                .arg(lost_keys)
                .arg(missing_keys)
                .output()
                .map_err(|e| e.to_string())?,
        };

        // Early exit if everything is okay and there's no output to show
        if !self.show_output && output.status.success() {
            return Ok(None);
        }

        const SEP: &str = "\n====================\n";

        // Compose a string with both stdout and stderr in it, separated by a divider
        let mut message = String::new();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

        if !stdout.is_empty() {
            message.push_str(&stdout);

            if !stderr.is_empty() {
                message.push_str(SEP);
            }
        }

        if !stderr.is_empty() {
            message.push_str(&stderr);
        }

        if output.status.success() {
            if message.is_empty() {
                Ok(None)
            } else {
                Ok(Some(message))
            }
        } else {
            Err(message)
        }
    }

    /// Performs a sanity check on the backend's configuration, specifically
    /// on the command string.
    ///
    /// # Returns
    /// - `Ok(())` if the sanity check passed without any issues.
    /// - `Err(Vec<String>)` if there were issues found during the sanity check,
    ///   containing a vector of descriptive error messages for each issue found.
    fn sanity_check(&self) -> Result<(), Vec<String>> {
        let mut vec = Vec::new();

        if self.command.trim().is_empty() {
            vec.push("Command string must not be empty".to_string());
        }

        if vec.is_empty() { Ok(()) } else { Err(vec) }
    }
}

/// Formats a list of keys and their corresponding timestamps into a
/// comma-separated string in the format "`key1:timestamp1,key2:timestamp2,...`".
///
/// # Parameters
/// - `peers`: A map of `wireguard::PeerKey` to their corresponding `wireguard::WireGuardPeer`
///   information, which includes the last seen timestamp for each peer.
/// - `keys`: A slice of keys for which to format the key-timestamp pairs.
///
/// # Returns
/// A comma-separated string of key-timestamp pairs in the format "`key:timestamp`".
fn format_key_timestamp_pairs(
    peers: &collections::HashMap<wireguard::PeerKey, wireguard::WireGuardPeer>,
    keys: &[wireguard::PeerKey],
) -> String {
    keys.iter()
        .filter_map(|key| {
            peers
                .get(key)
                .map(|peer| format!("{key}:{}", peer.last_seen_unix))
        })
        .collect::<Vec<String>>()
        .join(",")
}

#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use super::*;

    /// Tests the `format_key_timestamp_pairs` function to ensure it correctly
    /// formats the key-timestamp pairs as a comma-separated string.
    #[test]
    fn test_format_key_timestamp_pairs() {
        let mut peers = collections::HashMap::new();

        let key1_str = "vfpuUkQqZVkwZx1qvUkqcS+5PzqFqpWVQUO3nK3HXUk=";
        let key2_str = "PL5QAuDP8bM62q85P7YW+M5cz2WilbtKN6LDKhLRXCM=";

        let mut peer1 = wireguard::WireGuardPeer::new(key1_str, Some("Peer 1")).unwrap();
        peer1.last_seen_unix = 1234567890;
        peers.insert(peer1.public_key.clone(), peer1.clone());

        let mut peer2 = wireguard::WireGuardPeer::new(key2_str, Some("Peer 2")).unwrap();
        peer2.last_seen_unix = 9876543210;
        peers.insert(peer2.public_key.clone(), peer2.clone());

        let keys = vec![peer1.public_key.clone(), peer2.public_key.clone()];
        let result = format_key_timestamp_pairs(&peers, &keys);
        assert_eq!(
            result,
            "vfpuUkQqZVkwZx1qvUkqcS+5PzqFqpWVQUO3nK3HXUk=:1234567890,\
            PL5QAuDP8bM62q85P7YW+M5cz2WilbtKN6LDKhLRXCM=:9876543210"
        );

        let keys: Vec<wireguard::PeerKey> = Vec::new();
        let result = format_key_timestamp_pairs(&peers, &keys);
        assert_eq!(result, "");
    }
}
