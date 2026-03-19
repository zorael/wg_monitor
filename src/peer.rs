//! WireGuard peer presentation and management functionality.

use std::cmp;
use std::collections;
use std::time;

/// Represents a WireGuard peer, including its public key, human-readable name,
/// and last seen timestamp.
#[derive(Clone, Debug)]
pub struct WireGuardPeer {
    /// The public key of the WireGuard peer, which serves as its unique identifier.
    pub public_key: String,

    /// A human-readable name for the peer, which can be specified in the peer
    /// list file or derived from the public key if no name is provided.
    pub human_name: String,

    /// The last time the peer was seen as active, represented as an optional
    /// `SystemTime`. This is updated based on the output of the `wg show`
    /// command, and is used to determine if a peer is considered lost based on
    /// the configured timeout.
    pub last_seen: Option<time::SystemTime>,

    /// The timestamp of the last handshake with the peer, represented as an
    /// `u64` UNIX timestamp.
    pub last_seen_unix: u64,
}

impl WireGuardPeer {
    /// Shortens a WireGuard public key for display purposes, returning the
    /// first 7 characters, or the substring before a '/' or '+' if present in
    /// the first 7 characters. If the very first letter (index 0) is a
    /// '/' or '+', the match is ignored and the first 7 characters are returned.
    pub fn shorten_key(public_key: &str) -> String {
        fn check_for_delimiter(key: &str, delimiter: char) -> Option<String> {
            if let Some(pos) = key.find(delimiter)
                && pos > 0
            {
                let pre_delimiter = &key[..pos];
                return Some(pre_delimiter.to_string());
            }

            None
        }

        // We should not need this; validate_public_key ensures the key is 44
        // characters long. Keep it just in case.
        if public_key.len() < 7 {
            return public_key.to_string();
        }

        let first_seven = &public_key[..7];

        if let Some(shortened) = check_for_delimiter(first_seven, '/') {
            return shortened;
        }

        if let Some(shortened) = check_for_delimiter(first_seven, '+') {
            return shortened;
        }

        first_seven.to_string()
    }

    /// Validates a WireGuard public key, returning true if it does not seem
    /// obviously invalid. Does not perform an actual cryptographic validation.
    pub fn validate_public_key(public_key: &str) -> bool {
        const EXPECTED_LENGTH: usize = 44;

        if public_key.len() != EXPECTED_LENGTH || !public_key.ends_with('=') {
            return false;
        }

        public_key[..EXPECTED_LENGTH - 1] // skip trailing '=', already established above
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/')
    }

    /// Resets the last seen timestamps for the peer, setting `last_seen` to `None`
    /// and `last_seen_unix` to 0.
    pub fn reset_last_seen(&mut self) {
        self.last_seen = None;
        self.last_seen_unix = 0;
    }
}

/// Sorts an array of peer public keys based on their last seen UNIX timestamps in the
/// provided peers map. Peers that are present (have a non-0 timestamp) are sorted first,
/// with newer timestamps appearing before older ones. Peers without a timestamp
/// (or rather, with a timestamp of 0) are sorted last.
pub fn sort_keys(keys: &mut [String], peers: &collections::HashMap<String, WireGuardPeer>) {
    keys.sort_unstable_by_key(|k| {
        let timestamp = peers.get(k).map(|p| p.last_seen_unix).unwrap_or(0);
        (timestamp == 0, cmp::Reverse(timestamp))
    });
}
