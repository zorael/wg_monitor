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
    /// the first 7 characters.
    pub fn shorten_key(public_key: &str) -> String {
        if public_key.len() < 7 {
            return public_key.to_owned();
        }

        let key = &public_key[..7];

        if let Some(pos) = key.find('/') {
            let until_slash = &key[..pos];
            return until_slash.to_owned();
        }

        if let Some(pos) = key.find('+') {
            let until_plus = &key[..pos];
            return until_plus.to_owned();
        }

        key.to_owned()
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
