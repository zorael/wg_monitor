//! Defines the `WireGuardPeer` struct, which represents a peer machine in a
//! WireGuard VPN.

use std::borrow;
use std::cmp;
use std::collections;
use std::fmt;
use std::rc;
use std::time;

/// Represents a WireGuard peer, including its public key, a human-readable name,
/// and timestamps for when it was last seen as active.
#[derive(Clone, Debug)]
pub struct WireGuardPeer {
    /// A `PeerKey` newtype of the WireGuard public key for the peer,
    /// which is a 44-character base64 string that uniquely identifies the peer
    /// in the WireGuard network.
    pub public_key: PeerKey,

    /// A human-readable name for the peer, which can be used for display purposes in
    /// notifications and logs.
    ///
    /// This is not a required field in WireGuard itself, but it is useful for
    /// identifying peers in notifications.
    pub human_name: String,

    /// The timestamp of the last time the peer was seen as active, represented
    /// as an `Option<time::SystemTime>`.
    ///
    /// This can be `None` if the peer has never been seen or if the timestamp
    /// has been reset to 0 in a VPN restart.
    pub last_seen: Option<time::SystemTime>,

    /// The last seen timestamp represented as a UNIX timestamp (seconds since
    /// the UNIX epoch).
    pub last_seen_unix: u64,
}

impl WireGuardPeer {
    /// Creates a new `WireGuardPeer` instance from a public key string and an
    /// optional human-readable name.
    ///
    /// # Parameters
    /// - `public_key`: The WireGuard public key for the peer, which must pass
    ///   validation in `PeerKey::new`, else this function will return `None`.
    /// - `human_name`: An optional human-readable name for the peer.
    ///   If `None` is provided, the human name will be derived from the public
    ///   key using the `shorten_key` function.
    ///
    /// # Returns
    /// - `Some(WireGuardPeer)` if the provided public key is (seemingly) valid.
    /// - `None` if the public key is invalid.
    pub fn new(public_key: &str, human_name: Option<&str>) -> Option<Self> {
        let key = PeerKey::new(public_key)?;

        Some(Self {
            public_key: key,
            human_name: human_name
                .map(|s| s.to_string())
                .unwrap_or_else(|| Self::shorten_key(public_key)),
            last_seen: None,
            last_seen_unix: 0,
        })
    }

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
    /// A valid WireGuard public key is a 44-character Base64 string that ends
    /// with an '`=`' character. The function checks the length of the key,
    /// ensures it ends with '`=`', and verifies that all characters (except the
    /// trailing '`=`') are valid Base64 characters (alphanumeric, '`+`', '`/`').
    ///
    /// # Parameters
    /// - `public_key`: The WireGuard public key to validate.
    ///
    /// # Returns
    /// `true` if the public key seems valid, `false` otherwise.
    pub fn roughly_validate_public_key(public_key: &str) -> bool {
        const EXPECTED_LENGTH: usize = 44;

        if public_key.len() != EXPECTED_LENGTH || !public_key.ends_with('=') {
            return false;
        }

        public_key[..EXPECTED_LENGTH - 1] // skip trailing '=', already established above
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/')
    }

    /// Resets the last seen timestamps for the peer.
    pub fn reset_last_seen(&mut self) {
        self.last_seen = None;
        self.last_seen_unix = 0;
    }

    /// Sets the last seen timestamps for the peer based on a provided UNIX timestamp.
    pub fn set_last_seen(&mut self, seconds: u64) {
        self.last_seen_unix = seconds;
        self.last_seen = Some(time::UNIX_EPOCH + time::Duration::from_secs(seconds));
    }
}

/// Sorts an array of peer public keys based on their last seen UNIX timestamps
/// in the provided peers map.
///
/// Peers that are present (have a non-0 timestamp) are sorted first, with newer
/// timestamps appearing before older ones. Peers without a timestamp
/// (or rather, with a timestamp of 0) are sorted last.
///
/// # Parameters
/// - `keys`: A mutable slice of `PeerKey` instances representing the public
///   keys of peers to be sorted.
/// - `peers`: A reference to a hashmap mapping `PeerKey` instances to
///   `WireGuardPeer` instances, which is used to look up the last seen
///   timestamps for sorting the keys.
pub fn sort_keys(keys: &mut [PeerKey], peers: &collections::HashMap<PeerKey, WireGuardPeer>) {
    keys.sort_unstable_by_key(|k| {
        let timestamp = peers.get(k).map(|p| p.last_seen_unix).unwrap_or(0);
        (timestamp == 0, cmp::Reverse(timestamp))
    });
}

/// Newtype of `rc::Rc<str>` representing a WireGuard peer's public key,
/// used as a key in hashmaps and for display purposes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PeerKey(rc::Rc<str>);

impl PeerKey {
    /// Creates a new `PeerKey` from a string slice, "validating" that the input
    /// is a valid WireGuard public key.
    ///
    /// # Parameters
    /// - `key`: The string slice representing the WireGuard public key to be
    ///   converted into a `PeerKey`.
    ///
    /// # Returns
    /// `Some(PeerKey)` if the input string is a valid WireGuard public key,
    /// or `None` if it is invalid.
    pub fn new(key: &str) -> Option<Self> {
        if WireGuardPeer::roughly_validate_public_key(key) {
            Some(Self(rc::Rc::from(key)))
        } else {
            None
        }
    }

    /// Returns a string slice that represents the WireGuard public key contained in
    /// this `PeerKey`.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PeerKey {
    /// Formats the `PeerKey` for display purposes by passing it the inner string.
    ///
    /// # Parameters
    /// - `f`: The formatter to write the output to.
    ///
    /// # Returns
    /// An `fmt::Result` indicating whether the formatting was successful.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl borrow::Borrow<str> for PeerKey {
    /// Allows borrowing a `str` from a `PeerKey`.
    fn borrow(&self) -> &str {
        &self.0
    }
}
