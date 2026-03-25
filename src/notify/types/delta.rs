//! Delta struct representing the changes in peer status between two checks,
//! key in composing notification messages based on what changed since the
//! last check.
//!
//! This struct is computed from the `Context` and contains vectors of public
//! keys for peers that changed status, categorized by the type of change
//! (became late, went missing, no longer late, returned).

use crate::notify;
use crate::peer;
use crate::utils;

/// Delta struct representing the changes in peer status between two checks,
/// key in composing notification messages based on what changed since the
/// last check.
#[derive(Debug, Clone)]
pub struct Delta {
    /// Public keys of peers that were lost (time since last seen exceeds the
    /// timeout threshold) since the last check.
    pub became_late_keys: Vec<peer::PeerKey>,

    /// Public keys of peers that went missing (not seen at all) since the last check.
    /// This is indicative of a VPN restart.
    pub went_missing_keys: Vec<peer::PeerKey>,

    /// Public keys of peers that returned (time since last seen is now within
    /// the timeout threshold) since the last check.
    pub no_longer_late_keys: Vec<peer::PeerKey>,

    /// Public keys of peers that appeared after being missing (not seen at all)
    /// since the last check.
    pub returned_keys: Vec<peer::PeerKey>,
}

impl Delta {
    /// Creates a new `Delta` with the specified capacity for the key vectors.
    ///
    /// # Parameters
    /// - `capacity`: The capacity to use for the key vectors, which can help
    ///   avoid unnecessary allocations if the number of peers is known in advance.
    ///
    /// # Returns
    /// A new `Delta` instance with the specified capacity for the key vectors,
    /// initialized with empty vectors.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            became_late_keys: Vec::with_capacity(capacity),
            went_missing_keys: Vec::with_capacity(capacity),
            no_longer_late_keys: Vec::with_capacity(capacity),
            returned_keys: Vec::with_capacity(capacity),
        }
    }

    /// Clears all the key vectors in the `Delta`, effectively resetting it to an
    /// empty state while retaining the allocated capacity.
    pub fn clear(&mut self) {
        self.became_late_keys.clear();
        self.went_missing_keys.clear();
        self.no_longer_late_keys.clear();
        self.returned_keys.clear();
    }

    /// Returns whether all the key vectors in the `Delta` are empty, indicating
    /// that there are no changes in peer status since the last check.
    ///
    /// # Returns
    /// `true` if all key vectors are empty, and `false` if any of them contain keys.
    pub fn is_empty(&self) -> bool {
        self.became_late_keys.is_empty()
            && self.went_missing_keys.is_empty()
            && self.no_longer_late_keys.is_empty()
            && self.returned_keys.is_empty()
    }

    /// Computes the `Delta` from the given `Context`, determining which peers
    /// changed status since the last check and categorizing them into the
    /// appropriate vectors.
    ///
    /// # Parameters
    /// - `ctx`: The `Context` containing the current and previous state of peers.
    pub fn compute_from(&mut self, ctx: &notify::Context) {
        self.clear();

        utils::append_vec_difference(
            &ctx.previous_late_keys,
            &ctx.late_keys,
            &mut self.no_longer_late_keys,
            &mut self.became_late_keys,
        );

        utils::append_vec_difference(
            &ctx.previous_missing_keys,
            &ctx.missing_keys,
            &mut self.returned_keys,
            &mut self.went_missing_keys,
        );

        // Sort keys so that notifications present them in a descending order of
        // disappearance time, with missing peers last.
        peer::sort_keys(&mut self.no_longer_late_keys, &ctx.peers);
        peer::sort_keys(&mut self.became_late_keys, &ctx.peers);
        peer::sort_keys(&mut self.returned_keys, &ctx.peers);
        peer::sort_keys(&mut self.went_missing_keys, &ctx.peers);
    }

    /// Prints the non-empty key vectors in the `Delta` with a specified prefix
    /// for each line, useful for debugging or logging the changes in peer status.
    ///
    /// # Parameters
    /// - `prefix`: A string prefix to prepend to each line of output, which can
    ///   help visually distinguish this output in terminal output.
    pub fn print_nonempty_prefixed(&self, prefix: &str) {
        if !self.no_longer_late_keys.is_empty() {
            println!("{prefix}no longer late: {:?}", self.no_longer_late_keys);
        }
        if !self.became_late_keys.is_empty() {
            println!("{prefix}became late: {:?}", self.became_late_keys);
        }
        if !self.returned_keys.is_empty() {
            println!("{prefix}returned: {:?}", self.returned_keys);
        }
        if !self.went_missing_keys.is_empty() {
            println!("{prefix}went missing: {:?}", self.went_missing_keys);
        }
    }
}
