//! Module responsible for formatting notification messages based on the current
//! context and changes in peer status.

use std::collections;

use crate::peer;
use crate::settings;
use crate::utils;

/// Builds a generic notification message based on the provided `Context` and
/// `Delta`, using the specified message strings for formatting.
pub fn format_generic_message(
    ctx: &super::Context,
    delta: &super::Delta,
    strings: &settings::MessageStrings,
) -> String {
    let mut message = String::new();

    if ctx.first_run && !ctx.resume {
        // This is only called if there are actually missing or late peers,
        // so we don't need to check for whether missing_keys and late_keys are empty here
        message.push_str(&strings.first_run_missing);
        message.push('\n');

        let bp = &strings.bullet_point;

        for key in ctx.missing_keys.iter().chain(ctx.late_keys.iter()) {
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

        return message.replace("\\n", "\n").trim_end().to_owned();
    }

    let mut add_section = |keys: &[String], header: &str| {
        append_message_section(
            &ctx.peers,
            &mut message,
            keys,
            header,
            &strings.peer_with_timestamp,
            &strings.peer_no_timestamp,
            &strings.bullet_point,
        );
    };

    if ctx.resume {
        add_section(&ctx.late_keys, &strings.still_lost);
        add_section(&ctx.missing_keys, &strings.still_missing);
        message.push_str(&strings.footer);
        return message.replace("\\n", "\n").trim_end().to_owned();
    }

    let late_sans_new_late_keys =
        utils::get_elements_not_in_other_vec(&ctx.late_keys, &delta.became_late_keys);

    let missing_sans_new_missing_keys =
        utils::get_elements_not_in_other_vec(&ctx.missing_keys, &delta.went_missing_keys);

    add_section(&delta.became_late_keys, &strings.lost);
    add_section(&delta.went_missing_keys, &strings.forgot);
    add_section(&delta.no_longer_late_keys, &strings.returned);
    add_section(&delta.returned_keys, &strings.appeared);
    add_section(&late_sans_new_late_keys, &strings.still_lost);
    add_section(&missing_sans_new_missing_keys, &strings.still_missing);

    if !strings.footer.is_empty() {
        message.push('\n');
        message.push_str(&strings.footer);
    }

    message.replace("\\n", "\n").trim_end().to_owned()
}

/// Builds a generic reminder message based on the provided `Context`, using the
/// specified message strings for formatting.
pub fn format_generic_reminder(
    ctx: &super::Context,
    strings: &settings::ReminderStrings,
) -> String {
    let mut message = String::new();

    let mut add_section = |keys: &[String], header: &str| {
        append_message_section(
            &ctx.peers,
            &mut message,
            keys,
            header,
            &strings.peer_with_timestamp,
            &strings.peer_no_timestamp,
            &strings.bullet_point,
        );
    };

    add_section(&ctx.late_keys, &strings.still_lost);
    add_section(&ctx.missing_keys, &strings.still_missing);

    if !strings.footer.is_empty() {
        message.push('\n');
        message.push_str(&strings.footer);
    }

    message.replace("\\n", "\n").trim_end().to_owned()
}

/// Formats a single peer line for notifications, using the provided string
/// patterns for peers with and without timestamps.
fn format_peer_line(
    peer: &peer::WireguardPeer,
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
        .replace("{key}", &peer.public_key)
        .replace("{when}", &when)
        .replace("{timestamp}", &peer.timestamp.unwrap_or(0).to_string())
}

/// Appends a section to the notification message for a list of peer keys,
/// using the provided section header and patterns for formatting each peer line.
fn append_message_section(
    peers: &collections::HashMap<String, peer::WireguardPeer>,
    message: &mut String,
    keys: &[String],
    header: &str,
    peer_with_timestamp: &str,
    peer_no_timestamp: &str,
    bullet_point: &str,
) {
    if keys.is_empty() || header.is_empty() {
        return;
    }

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
