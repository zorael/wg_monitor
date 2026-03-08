//! Module defining the `Delta` struct, which represents the changes in peer
//! status between two checks, containing vectors of public keys for peers that
//! became late, went missing, are no longer late or returned after being missing.
//!
//! The `Delta` is computed based on the current and previous peer status in
//! the `Context` struct.

use crate::notify;
use crate::utils;

/// Structure representing the changes in peer status between two checks.
#[derive(Debug, Clone)]
pub struct Delta {
    /// Public keys of peers that were lost (time since last seen exceeds the
    /// timeout threshold) since the last check.
    pub became_late_keys: Vec<String>,

    /// Public keys of peers that went missing (not seen at all) since the last check.
    /// This is indicative of a VPN restart.
    pub went_missing_keys: Vec<String>,

    /// Public keys of peers that returned (time since last seen is now within
    /// the timeout threshold) since the last check.
    pub no_longer_late_keys: Vec<String>,

    /// Public keys of peers that appeared after being missing (not seen at all)
    /// since the last check.
    pub returned_keys: Vec<String>,
}

impl Delta {
    /// Creates a new `Delta` with the specified capacity for the key vectors.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            became_late_keys: Vec::with_capacity(capacity),
            went_missing_keys: Vec::with_capacity(capacity),
            no_longer_late_keys: Vec::with_capacity(capacity),
            returned_keys: Vec::with_capacity(capacity),
        }
    }

    /// Clears all the key vectors in the `Delta`, preparing it for reuse.
    pub fn clear(&mut self) {
        self.became_late_keys.clear();
        self.went_missing_keys.clear();
        self.no_longer_late_keys.clear();
        self.returned_keys.clear();
    }

    /// Checks if the `Delta` is empty, indicating there were no changes in peer status.
    pub fn is_empty(&self) -> bool {
        self.became_late_keys.is_empty()
            && self.went_missing_keys.is_empty()
            && self.no_longer_late_keys.is_empty()
            && self.returned_keys.is_empty()
    }

    /// Computes the `Delta` based on the current and previous peer status in
    /// the provided `Context`, populating the vectors with the appropriate
    /// public keys for each category of change.
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
    }

    /// Prints the non-empty vectors in the `Delta` for debugging purposes,
    /// showing which peers changed status.
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
