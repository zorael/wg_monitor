//! Wireguard peer presentation and management functionality.

use std::time;

/// Represents a Wireguard peer, including its public key, human-readable name,
/// and last seen timestamp.
#[derive(Clone, Debug)]
pub struct WireguardPeer {
    /// The public key of the Wireguard peer, which serves as its unique identifier.
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
    /// optional `u64` UNIX timestamp.
    pub timestamp: Option<u64>,
}

impl WireguardPeer {
    /// Shortens a Wireguard public key for display purposes, returning the
    /// first 7 characters, or the substring before a '/' or '+' if present in
    /// the first 7 characters. If the very first letter (index 0) is a
    /// '/' or '+', the match is ignored and the first 7 characters are returned.
    pub fn shorten_key(public_key: &str) -> String {
        fn check_for_delimiter(key: &str, delimiter: char) -> Option<String> {
            if let Some(pos) = key.find(delimiter)
                && pos > 0
            {
                let pre_delimiter = &key[..pos];
                return Some(pre_delimiter.to_owned());
            }

            None
        }

        // We should not need this; validate_public_key ensures the key is 44
        // characters long. Keep it just in case.
        if public_key.len() < 7 {
            return public_key.to_owned();
        }

        let first_seven = &public_key[..7];

        if let Some(shortened) = check_for_delimiter(first_seven, '/') {
            return shortened;
        }

        if let Some(shortened) = check_for_delimiter(first_seven, '+') {
            return shortened;
        }

        first_seven.to_owned()
    }

    /// Validates a Wireguard public key, returning true if it does not seem
    /// obviously invalid. Does not perform an actual cryptographic validation.
    pub fn validate_public_key(public_key: &str) -> bool {
        if public_key.len() != 44 || !public_key.ends_with('=') {
            return false;
        }

        public_key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
    }
}
