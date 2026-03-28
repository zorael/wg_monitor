//! Context struct for notification message-composing.

use std::collections;
use std::mem;
use std::path;
use std::time;

use crate::wireguard;

#[derive(Clone, Debug)]
/// Context struct for notification message-composing, containing the current
/// and previous state of peers.
pub struct Context {
    /// Current peers, keyed by their public key (in form of a `wireguard::PeerKey`).
    ///
    /// Can be used by notification backends to access peer information when
    /// composing notifications.
    pub peers: collections::HashMap<wireguard::PeerKey, wireguard::WireGuardPeer>,

    /// Current peers that are lost; they have not been seen in the last
    /// timeout duration, but they have been seen at some point in the past.
    pub lost_keys: Vec<wireguard::PeerKey>,

    /// Current peers that are missing; they have not been seen at all since
    /// the VPN started (or restarted).
    pub missing_keys: Vec<wireguard::PeerKey>,

    /// Peers that were previously lost in the last check.
    ///
    /// They still might be; this is just a clone of the previous state.
    pub previous_lost_keys: Vec<wireguard::PeerKey>,

    /// Peers that were previously missing in the last check.
    ///
    /// They still might be; this is just a clone of the previous state.
    pub previous_missing_keys: Vec<wireguard::PeerKey>,

    /// The current time.
    pub now: time::SystemTime,

    /// The current loop iteration count, used to
    /// indicate how many times the program has checked the peers since it started.
    ///
    /// This starts at 0 for the first run, and increments by 1 on each loop iteration.
    /// If `--resume` was passed at the command line, this will start at 1 instead.
    pub loop_iteration: usize,

    /// Whether or not the program is resuming from a previous run, used to
    /// prevent the program from sending an initial first-run "program started" notification.
    pub resume: bool,

    /// The path to the peer list file, which can be used by some notification
    /// backends for reading peers' human-readable names.
    ///
    /// # Notes
    /// This is currently only used by the Command notification backend, but it
    /// could potentially be used by other backends in the future if needed.
    ///
    /// It arguably does not really belong in this struct.
    pub peer_list: path::PathBuf,
}

impl Context {
    /// Creates a new `Context` with the specified capacity for the peer-related vectors.
    ///
    /// # Parameters
    /// - `capacity`: The capacity to use for the peer-related vectors,
    ///   which helps avoid unnecessary allocations if the number of peers
    ///   is known in advance (which is the case in the current implementation).
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
            peer_list: path::PathBuf::new(),
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
    /// the `previous_*` fields, and clearing the current lost and missing keys
    /// for the next check.
    pub fn rotate(&mut self) {
        mem::swap(&mut self.lost_keys, &mut self.previous_lost_keys);
        mem::swap(&mut self.missing_keys, &mut self.previous_missing_keys);
        self.lost_keys.clear();
        self.missing_keys.clear();
    }

    /// Returns `true` if this is the first run of the program (loop iteration
    /// count is zero), and `false` otherwise.
    pub fn is_first_run(&self) -> bool {
        self.loop_iteration == 0
    }
}
