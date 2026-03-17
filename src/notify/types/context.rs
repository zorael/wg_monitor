//! Module defining the `Context` struct, which holds the state of peers and timing information
//! used for building notification messages in the `notify` module.

use std::collections;
use std::mem;
use std::time;

use crate::peer;

#[derive(Clone, Debug)]
/// Context for building notification messages, containing the current and previous
/// state of peers, as well as timing information.
pub struct Context {
    /// Map of public keys to `WireGuardPeer` structs, representing all known peers.
    pub peers: collections::HashMap<String, peer::WireGuardPeer>,

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

    /// The current loop iteration, indicating whether this is the first run of
    /// the notification loop, which can be used to adjust messaging and behavior accordingly.
    pub loop_iteration: usize,

    /// Indicates that the program is resuming from a previous run, which means
    /// that some startup notifications should be skipped.
    pub resume: bool,

    /// The file path of the peer list file, which can be used in notifications
    /// to indicate the source of the peer information.
    pub peer_list_file_path: String,
}

impl Context {
    /// Creates a new `Context` with the specified capacity for the peer key vectors.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            peers: collections::HashMap::with_capacity(capacity),
            late_keys: Vec::with_capacity(capacity),
            missing_keys: Vec::with_capacity(capacity),
            previous_late_keys: Vec::with_capacity(capacity),
            previous_missing_keys: Vec::with_capacity(capacity),
            now: time::SystemTime::UNIX_EPOCH,
            loop_iteration: 0,
            resume: false,
            peer_list_file_path: String::new(),
        }
    }

    /// Creates a new `Context` with the provided peers, initializing the
    /// vectors based on the number of peers.
    pub fn inherit(peers: collections::HashMap<String, peer::WireGuardPeer>) -> Self {
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

    /// Whether or not this is the first run; e.g. `loop_iteration` is 0.
    pub fn is_first_run(&self) -> bool {
        self.loop_iteration == 0
    }
}
