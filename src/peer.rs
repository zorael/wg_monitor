//! Defines the `WireGuardPeer` struct, which represents a WireGuard peer and
//! includes methods for validating and shortening public keys, as well as a
//! function for sorting peer keys based on their last seen timestamps.

use std::cmp;
use std::collections;
use std::time;

/// Represents a WireGuard peer, including its public key, human-readable name,
/// and timestamps for when it was last seen as active.
#[derive(Clone, Debug)]
pub struct WireGuardPeer {
    /// The WireGuard public key for the peer, which is a 44-character base64 string
    /// that uniquely identifies the peer in the WireGuard network.
    pub public_key: String,

    /// A human-readable name for the peer, which can be used for display purposes in
    /// notifications and logs.
    ///
    /// This is not a required field in WireGuard itself, but it can be set
    /// based on the configuration or other metadata to make it easier to
    /// identify peers in notifications.
    pub human_name: String,

    /// The timestamp of the last time the peer was seen as active, represented
    /// as an `Option<SystemTime>`.
    ///
    /// This can be `None` if the peer has never been seen or if the timestamp
    /// has been reset.
    pub last_seen: Option<time::SystemTime>,

    /// The last seen timestamp represented as a UNIX timestamp (seconds since
    /// the UNIX epoch). This is used for easier sorting and comparison of peers
    /// based on their last seen times.
    pub last_seen_unix: u64,
}

impl WireGuardPeer {
    /// Shortens a WireGuard public key for display purposes.
    ///
    /// The function takes a public key string and returns a shortened
    /// version of it, which is useful for displaying in notifications without
    /// showing the full key when no human-readable name has been set.
    ///
    /// The shortening logic looks for common delimiters ('/' and '+')
    /// in the first 7 characters of the key. If a delimiter is found and it is
    /// not the very first character, the substring before the delimiter is
    /// returned. Otherwise, the first 7 characters are returned.
    ///
    /// # Parameters
    /// - `public_key`: The full WireGuard public key to be shortened.
    ///
    /// # Returns
    /// A shortened version of the public key, suitable for display in notifications.
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

    /// "Validates" a WireGuard public key to ensure it is in the correct format.
    ///
    /// A valid WireGuard public key is a 44-character base64 string that ends
    /// with an '=' character. The function checks the length of the key,
    /// ensures it ends with '=', and verifies that all characters (except the
    /// trailing '=') are valid base64 characters (alphanumeric, '+', '/').
    ///
    /// # Parameters
    /// - `public_key`: The WireGuard public key to validate.
    ///
    /// # Returns
    /// `true` if the public key seems valid, `false` otherwise.
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
/// provided peers map.
///
/// Peers that are present (have a non-0 timestamp) are sorted first, with newer
/// timestamps appearing before older ones. Peers without a timestamp
/// (or rather, with a timestamp of 0) are sorted last.
pub fn sort_keys(keys: &mut [String], peers: &collections::HashMap<String, WireGuardPeer>) {
    keys.sort_unstable_by_key(|k| {
        let timestamp = peers.get(k).map(|p| p.last_seen_unix).unwrap_or(0);
        (timestamp == 0, cmp::Reverse(timestamp))
    });
}
