//! This module contains functions for formatting notification messages based
//! on the `Context` and `KeyDelta` of peer status changes, using customizable message
//! strings defined in the settings.
//!
//! It provides generic formatting logic that can be reused across different
//! types of notifications and reminders. Each backend has its own configuration
//! for message strings, but they all utilize the same core formatting functions.

use std::collections;

use crate::defaults;
use crate::settings;
use crate::utils;
use crate::wireguard;

/// Builds a generic notification message based on the provided `Context` and
/// `KeyDelta`, using the specified message strings for formatting.
///
/// The message is composed of sections for peers that became lost, went missing,
/// returned, etc., with appropriate headers and formatting based on the provided
/// message strings.
///
/// # Note:
/// The returned `String` may be empty if there are no peers to report, or if the
/// message strings are configured in such a way that no message should be sent
/// (such as empty headers for some peer state where peer would have been listed).
///
/// # Parameters
/// - `ctx`: The notification context containing the current state of peers.
/// - `delta`: The delta containing the changes in peer status since the last check.
/// - `strings`: The message strings to use for formatting the notification.
///
/// # Returns
/// A formatted notification message as a `String`.
fn format_generic_message(
    ctx: &super::Context,
    delta: &super::KeyDelta,
    strings: &settings::MessageStrings,
) -> String {
    let mut message = String::new();

    if ctx.is_first_run() && !ctx.resume {
        if strings.first_run_missing.is_empty()
            || (ctx.missing_keys.is_empty() && ctx.lost_keys.is_empty())
        {
            return message.trim_end().to_string();
        }

        if !strings.first_run_missing.is_empty() {
            message.push_str(&strings.first_run_missing);
            message.push('\n');
        }

        let bp = &strings.bullet_point;

        for key in ctx.lost_keys.iter().chain(ctx.missing_keys.iter()) {
            if let Some(peer) = ctx.peers.get(key) {
                let line = format_peer_line(
                    peer,
                    &strings.peer_with_timestamp,
                    &strings.peer_no_timestamp,
                );
                message.push_str(&format!("{bp}{line}\n"));
            }
        }

        if !strings.footer.is_empty() {
            message.push('\n');
            message.push_str(&strings.footer);
        }

        return message.trim_end().to_string();
    }

    let mut add_section = |keys: &[wireguard::PeerKey], header: &str, disable_timestamps: bool| {
        append_message_section(
            &ctx.peers,
            &mut message,
            keys,
            header,
            &strings.peer_with_timestamp,
            &strings.peer_no_timestamp,
            &strings.bullet_point,
            disable_timestamps,
        );
    };

    if ctx.resume {
        add_section(&ctx.lost_keys, &strings.still_lost, false);
        add_section(&ctx.missing_keys, &strings.still_missing, false);

        if !strings.footer.is_empty() {
            message.push_str(&strings.footer);
        }

        return message.trim_end().to_string();
    }

    let lost_sans_now_lost_keys =
        utils::get_elements_not_in_other_vec(&ctx.lost_keys, &delta.now_lost);

    let missing_sans_now_missing_keys =
        utils::get_elements_not_in_other_vec(&ctx.missing_keys, &delta.now_missing);

    // Revisit this order.
    add_section(&delta.now_lost, &strings.lost, false);
    add_section(&delta.was_lost, &strings.returned, true);
    add_section(&delta.now_missing, &strings.forgot, false);
    add_section(&delta.was_missing, &strings.appeared, true);
    add_section(&lost_sans_now_lost_keys, &strings.still_lost, false);
    add_section(
        &missing_sans_now_missing_keys,
        &strings.still_missing,
        false,
    );

    if !strings.footer.is_empty() {
        //message.push('\n'); // append_message_section leaves an extra newline
        message.push_str(&strings.footer);
    }

    message.trim_end().to_string()
}

/// Builds a generic reminder message based on the provided `Context` and
/// message strings for reminders.
///
/// This is similar to `format_generic_message` but is used for reminder
/// notifications, which only report peers that are still lost or
/// missing since the last check, in cases where there are no peers that changed
/// status since the last notification.
///
/// # Parameters
/// - `ctx`: The notification context containing the current state of peers.
/// - `strings`: The message strings to use for formatting the reminder notification.
///
/// # Returns
/// A formatted reminder message as a `String`.
fn format_generic_reminder(ctx: &super::Context, strings: &settings::ReminderStrings) -> String {
    let mut message = String::new();

    let mut add_section = |keys: &[wireguard::PeerKey], header: &str| {
        append_message_section(
            &ctx.peers,
            &mut message,
            keys,
            header,
            &strings.peer_with_timestamp,
            &strings.peer_no_timestamp,
            &strings.bullet_point,
            false,
        );
    };

    add_section(&ctx.lost_keys, &strings.still_lost);
    add_section(&ctx.missing_keys, &strings.still_missing);

    if !strings.footer.is_empty() {
        //message.push('\n'); // append_message_section leaves an extra newline
        message.push_str(&strings.footer);
    }

    message.trim_end().to_string()
}

/// Formats a single peer line for the notification message based on the peer's
/// information and the provided patterns for peers with and without timestamps.
///
/// The pattern can include placeholders `{peer}`, `{key}`, `{when}`,
/// `{unix}` and `{version}`, which will be replaced with the peer's human-readable name,
/// public key, last seen time (formatted), last seen time (unix timestamp),
/// and the program version respectively.
///
/// # Parameters
/// - `peer`: The `WireGuardPeer` whose information is to be formatted into a
///   line in the notification message.
/// - `pattern_with_timestamp`: The pattern to use for formatting if the peer
///   has a known last seen time (i.e., is "lost").
/// - `pattern_without_timestamp`: The pattern to use for formatting if the
///   peer does not have a known last seen time (i.e., is "missing").
///
/// # Returns
/// A formatted string representing the peer line in the notification message.
fn format_peer_line(
    peer: &wireguard::WireGuardPeer,
    pattern_with_timestamp: &str,
    pattern_without_timestamp: &str,
) -> String {
    let pattern = match peer.last_seen {
        Some(_) => pattern_with_timestamp,
        None => pattern_without_timestamp,
    };

    let when = match peer.last_seen {
        Some(ts) => {
            let dt: chrono::DateTime<chrono::Local> = ts.into();
            dt.format("%Y-%m-%d %H:%M:%S").to_string()
        }
        None => "never".to_string(),
    };

    String::from(pattern)
        .replace("{peer}", &peer.human_name)
        .replace("{key}", peer.public_key.as_str())
        .replace("{when}", &when)
        .replace("{unix}", &peer.last_seen_unix.to_string())
        .replace("{version}", defaults::program_metadata::VERSION)
}

/// Appends a section to the notification message for a list of peer keys, using
/// the specified header and formatting options.
///
/// This is a helper function used by both `format_generic_message` and
/// `format_generic_reminder` to avoid code duplication when adding sections for
/// different categories of peers (such as lost, missing, still lost, etc.).
///
/// # Parameters
/// - `peers`: A hashmap of all peers, keyed by `wireguard::PeerKey` instances, used
///   to look up peer information for formatting.
/// - `message`: The message string being composed, to which the section will be appended.
/// - `keys`: The list of peer public keys that belong to this section (such as
///   "lost" peers, "missing" peers, etc.).
/// - `header`: The header string for this section, which will be added before listing the peers.
/// - `peer_with_timestamp`: The pattern to use for formatting peers with a known last seen
///   time ("lost" peers and "forgotten" peers).
/// - `peer_no_timestamp`: The pattern to use for formatting peers without a known last seen
///   time ("missing" peers, "returning" peers and "appearing" peers).
/// - `bullet_point`: The string to use as a bullet point for listing peers in this
///   section.
/// - `disable_timestamps`: A boolean indicating whether to disable timestamps in the
///   peer formatting, which is used to select between use of `peer_with_timestamp`
///   and `peer_no_timestamp` patterns for formatting peers in this section.
#[allow(clippy::too_many_arguments)]
fn append_message_section(
    peers: &collections::HashMap<wireguard::PeerKey, wireguard::WireGuardPeer>,
    message: &mut String,
    keys: &[wireguard::PeerKey],
    header: &str,
    peer_with_timestamp: &str,
    peer_no_timestamp: &str,
    bullet_point: &str,
    disable_timestamps: bool,
) {
    if keys.is_empty() || header.is_empty() {
        return;
    }

    let peer_with_timestamp = if disable_timestamps {
        peer_no_timestamp
    } else {
        peer_with_timestamp
    };

    message.push_str(header);
    message.push('\n');

    for key in keys {
        if let Some(peer) = peers.get(key) {
            let line = format_peer_line(peer, peer_with_timestamp, peer_no_timestamp);
            message.push_str(&format!("{bullet_point}{line}\n"));
        }
    }

    message.push('\n');
}

/// Prepares the message body for a notification by formatting it based on the
/// provided `Context`, `KeyDelta` and message strings, applying a header
/// closure to the appropriate header string.
///
/// The function unescapes and trims the final message before returning it.
///
/// # Parameters
/// - `ctx`: The notification context containing the current state of peers.
/// - `delta`: The key delta containing the changes in peer status since the last check.
/// - `strings`: The message strings to use for formatting the notification.
/// - `header_closure`: A closure that takes a header string and returns a
///   formatted header string, which allows for backend-specific header
///   formatting (such as prepending "Subject: " for email bodies).
///
/// # Returns
/// - `Some(String)` if a message to send was composed.
/// - `None` if an empty message was composed. This can happen if strings are
///   configured so that the section header strings for peers to be listed
///   were empty, disabling that section from being output. In this case,
///   no message will be sent.
pub fn prepare_message_body(
    ctx: &super::Context,
    delta: &super::KeyDelta,
    strings: &settings::MessageStrings,
    header_closure: impl Fn(&str) -> String,
) -> Option<String> {
    let mut message = String::new();
    let body = &format_generic_message(ctx, delta, strings);

    if body.is_empty() && !ctx.is_first_run() {
        // Nothing to send. If it's the first run, we still want to send the
        // "first run" banner, even if there are no changes.
        return None;
    }

    let header = match ctx.is_first_run() {
        true => &strings.first_run_header,
        false => &strings.header,
    };

    if !header.is_empty() {
        message.push_str(&header_closure(header));
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

/// Prepares the message body for a reminder notification by formatting it based
/// on the provided `Context` and message strings, and applying a header closure
/// to the appropriate header string.
///
/// The function unescapes and trims the final message before returning it.
///
/// # Parameters
/// - `ctx`: The reminder context containing the current state of peers.
/// - `strings`: The message strings to use for formatting the reminder notification.
/// - `header_closure`: A closure that takes a header string and returns a
///   formatted header string, which allows for backend-specific header
///   formatting (such as prepending "Subject: " for email bodies).
///
/// # Returns
/// - `Some(String)` if a message to send was composed.
/// - `None` if an empty message was composed. This can happen if strings are
///   configured so that the section header strings for peers to be listed
///   were empty, disabling that section from being output. In this case,
///   no message will be sent.
pub fn prepare_reminder_body(
    ctx: &super::Context,
    strings: &settings::ReminderStrings,
    header_closure: impl Fn(&str) -> String,
) -> Option<String> {
    let mut message = String::new();
    let body = &format_generic_reminder(ctx, strings);

    if body.is_empty() {
        return None;
    }

    if !strings.header.is_empty() {
        message.push_str(&header_closure(&strings.header));
        message.push('\n');
    }

    message.push_str(body);

    let message = utils::unescape(&message).trim_end().to_string();
    Some(message)
}
