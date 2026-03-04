//! Module defining the `Context` struct, which holds the state of peers and timing information
//! used for building notification messages in the `notify` module.

use std::collections::HashMap;
use std::mem;
use std::time;

use crate::peer;

/// Context for building notification messages, containing the current and previous
/// state of peers, as well as timing information.
pub struct Context {
    /// Map of public keys to `WireguardPeer` structs, representing all known peers.
    pub peers: HashMap<String, peer::WireguardPeer>,

    /// Current peers that are late (seen but not within the expected time).
    pub late_keys: Vec<String>,

    /// Current peers that are missing (not seen at all).
    pub missing_keys: Vec<String>,

    /// Peers that were previously late in the last check.
    pub previous_late_keys: Vec<String>,

    /// Peers that were previously missing in the last check.
    pub previous_missing_keys: Vec<String>,

    /// Current time.
    pub now: time::SystemTime,

    /// Time of the last report sent, used for scheduling reminder notifications.
    pub last_report: Option<time::SystemTime>,

    /// Number of consecutive reminder notifications sent.
    pub num_consecutive_reminders: u32,

    /// Indicates whether this is the first run of the notifier, which can be used
    /// to adjust the messaging (e.g., to indicate that the initial state is being reported).
    pub first_run: bool,

    /// Indicates that the program is resuming from a previous run, which means
    /// that some startup notifications should be skipped.
    pub resume: bool,
}

impl Context {
    /// Creates a new `Context` with the specified capacity for the peer key vectors.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            peers: HashMap::with_capacity(capacity),
            late_keys: Vec::with_capacity(capacity),
            missing_keys: Vec::with_capacity(capacity),
            previous_late_keys: Vec::with_capacity(capacity),
            previous_missing_keys: Vec::with_capacity(capacity),
            now: time::SystemTime::UNIX_EPOCH,
            last_report: None,
            num_consecutive_reminders: 0,
            first_run: false,
            resume: false,
        }
    }

    /// Creates a new `Context` with the provided peers, initializing the
    /// vectors based on the number of peers.
    pub fn inherit(peers: HashMap<String, peer::WireguardPeer>) -> Self {
        let mut sized = Self::with_capacity(peers.len());
        sized.peers = peers;
        sized
    }

    /// Rotates the current late and missing peer vectors into the previous vectors,
    /// clearing the current ones. They retain their capacity.
    pub fn rotate(&mut self) {
        mem::swap(&mut self.late_keys, &mut self.previous_late_keys);
        mem::swap(&mut self.missing_keys, &mut self.previous_missing_keys);
        self.late_keys.clear();
        self.missing_keys.clear();
    }
}
