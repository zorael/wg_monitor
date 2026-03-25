//! Context struct for notification message-composing, containing the current
//! and previous state of peers, as well as timing information.

use std::collections;
use std::mem;
use std::time;

use crate::wireguard;

#[derive(Clone, Debug)]
/// Context struct for notification message-composing, containing the current
/// and previous state of peers, as well as timing information.
pub struct Context {
    /// Current peers, keyed by their public key.
    ///
    /// Can be used by notification backends to access peer information when
    /// composing notifications.
    pub peers: collections::HashMap<wireguard::PeerKey, wireguard::WireGuardPeer>,

    /// Current peers that are lost (seen but timed out).
    pub lost_keys: Vec<wireguard::PeerKey>,

    /// Current peers that are missing (not seen at all).
    pub missing_keys: Vec<wireguard::PeerKey>,

    /// Peers that were previously lost in the last check.
    pub previous_lost_keys: Vec<wireguard::PeerKey>,

    /// Peers that were previously missing in the last check.
    pub previous_missing_keys: Vec<wireguard::PeerKey>,

    /// The current time, which can be used in notifications to indicate when
    /// the notification is being sent, or to calculate durations since the
    /// last seen time of peers.
    pub now: time::SystemTime,

    /// The current loop iteration count, which can be used in notifications to
    /// indicate how many times the program has checked the peers since it started.
    ///
    /// This starts at 0 for the first run, and increments by 1 on each loop iteration.
    /// If `--resume` was passed, this will start at 1 instead.
    pub loop_iteration: usize,

    /// Whether or not the program is resuming from a previous run, which can be
    /// used in notifications to indicate that the program has been restarted
    /// and is resuming its checks.
    pub resume: bool,

    /// The path to the peer list file, which can be used by some notification
    /// backends for reading peers' human-readable names.
    pub peer_list_file_path: String,
}

impl Context {
    /// Creates a new `Context` with the specified capacity for the peer-related vectors.
    ///
    /// # Parameters
    /// - `capacity`: The capacity to use for the peer-related vectors,
    ///   which can help avoid unnecessary allocations if the number of peers
    ///   is known in advance.
    ///
    /// # Returns
    /// A new `Context` instance with the specified capacity for the
    /// peer-related vectors, initialized with default values for other fields.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            peers: collections::HashMap::with_capacity(capacity),
            lost_keys: Vec::with_capacity(capacity),
            missing_keys: Vec::with_capacity(capacity),
            previous_lost_keys: Vec::with_capacity(capacity),
            previous_missing_keys: Vec::with_capacity(capacity),
            now: time::SystemTime::UNIX_EPOCH,
            loop_iteration: 0,
            resume: false,
            peer_list_file_path: String::new(),
        }
    }

    /// Creates a new `Context` by inheriting the peer information from a previous state.
    ///
    /// # Parameters
    /// - `peers`: A hashmap of peers to inherit, keyed by their public keys.
    ///
    /// # Returns
    /// A new `Context` instance with the specified peers and default values
    /// for other fields.
    pub fn inherit(
        peers: collections::HashMap<wireguard::PeerKey, wireguard::WireGuardPeer>,
    ) -> Self {
        let mut sized = Self::with_capacity(peers.len());
        sized.peers = peers;
        sized
    }

    /// Rotates the peer state by moving the current lost and missing keys to
    /// the previous fields, and clearing the current lost and missing keys
    /// for the next check.
    pub fn rotate(&mut self) {
        mem::swap(&mut self.lost_keys, &mut self.previous_lost_keys);
        mem::swap(&mut self.missing_keys, &mut self.previous_missing_keys);
        self.lost_keys.clear();
        self.missing_keys.clear();
    }

    /// Returns whether this is the first run of the program, which is
    /// determined by whether the loop iteration count is zero.
    ///
    /// # Returns
    /// `true` if this is the first run of the program (loop iteration count
    /// is zero), and `false` otherwise.
    pub fn is_first_run(&self) -> bool {
        self.loop_iteration == 0
    }
}
