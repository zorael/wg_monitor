//! Delta struct representing the changes in peer status between two checks,
//! key in composing notification messages based on what changed since the
//! last check.
//!
//! This struct is computed from the `Context` and contains vectors of public
//! keys for peers that changed status, categorized by the type of change
//! (now lost, now missing, was lost, was missing).

use crate::notify;
use crate::utils;
use crate::wireguard;

/// Delta struct representing the changes in peer status between two checks,
/// key in composing notification messages based on what changed since the
/// last check.
#[derive(Debug, Clone)]
pub struct KeyDelta {
    /// Public keys of peers that were lost (time since last seen exceeds the
    /// timeout threshold) since the last check.
    pub now_lost: Vec<wireguard::PeerKey>,

    /// Public keys of peers that returned (time since last seen is now within
    /// the timeout threshold) since the last check.
    pub was_lost: Vec<wireguard::PeerKey>,

    /// Public keys of peers that went missing (not seen at all) since the last check.
    /// This is indicative of a VPN restart.
    pub now_missing: Vec<wireguard::PeerKey>,

    /// Public keys of peers that appeared after being missing (not seen at all)
    /// since the last check.
    pub was_missing: Vec<wireguard::PeerKey>,
}

impl KeyDelta {
    /// Creates a new `KeyDelta` with the specified capacity for the key vectors.
    ///
    /// # Parameters
    /// - `capacity`: The capacity to use for the key vectors, which can help
    ///   avoid unnecessary allocations if the number of peers is known in advance.
    ///
    /// # Returns
    /// A new `KeyDelta` instance with the specified capacity for the key vectors,
    /// initialized with empty vectors.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            now_lost: Vec::with_capacity(capacity),
            was_lost: Vec::with_capacity(capacity),
            now_missing: Vec::with_capacity(capacity),
            was_missing: Vec::with_capacity(capacity),
        }
    }

    /// Clears all the key vectors in the `KeyDelta`, effectively resetting it to an
    /// empty state while retaining the allocated capacity.
    pub fn clear(&mut self) {
        self.now_lost.clear();
        self.was_lost.clear();
        self.now_missing.clear();
        self.was_missing.clear();
    }

    /// Returns whether all the key vectors in the `KeyDelta` are empty, indicating
    /// that there are no changes in peer status since the last check.
    ///
    /// # Returns
    /// `true` if all key vectors are empty, and `false` if any of them contain keys.
    pub fn is_empty(&self) -> bool {
        self.now_lost.is_empty()
            && self.was_lost.is_empty()
            && self.now_missing.is_empty()
            && self.was_missing.is_empty()
    }

    /// Computes the `KeyDelta` from the given `Context`, determining which peers
    /// changed status since the last check and categorizing them into the
    /// appropriate vectors.
    ///
    /// # Parameters
    /// - `ctx`: The `Context` containing the current and previous state of peers.
    pub fn compute_from(&mut self, ctx: &notify::Context) {
        self.clear();

        utils::append_vec_difference(
            &ctx.previous_lost_keys,
            &ctx.lost_keys,
            &mut self.was_lost,
            &mut self.now_lost,
        );

        utils::append_vec_difference(
            &ctx.previous_missing_keys,
            &ctx.missing_keys,
            &mut self.was_missing,
            &mut self.now_missing,
        );

        // Sort keys so that notifications present them in a descending order of
        // disappearance time, with missing peers last.
        wireguard::sort_keys(&mut self.now_lost, &ctx.peers);
        wireguard::sort_keys(&mut self.was_lost, &ctx.peers);
        wireguard::sort_keys(&mut self.now_missing, &ctx.peers);
        wireguard::sort_keys(&mut self.was_missing, &ctx.peers);
    }

    /// Prints the non-empty key vectors in the `KeyDelta` with a specified prefix
    /// for each line, useful for debugging or logging the changes in peer status.
    ///
    /// # Parameters
    /// - `prefix`: A string prefix to prepend to each line of output, which can
    ///   help visually distinguish this output in terminal output.
    pub fn print_nonempty_prefixed(&self, prefix: &str) {
        if !self.now_lost.is_empty() {
            println!("{prefix}now lost: {:?}", self.now_lost);
        }
        if !self.was_lost.is_empty() {
            println!("{prefix}was lost: {:?}", self.was_lost);
        }
        if !self.now_missing.is_empty() {
            println!("{prefix}now missing: {:?}", self.now_missing);
        }
        if !self.was_missing.is_empty() {
            println!("{prefix}was missing: {:?}", self.was_missing);
        }
    }
}
