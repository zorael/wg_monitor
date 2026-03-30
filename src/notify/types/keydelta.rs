//! Delta struct representing the changes in peer status between two checks
//! (from a key perspective), instrumental in composing notification messages
//! based on what changed since the last check.
//!
//! This struct is computed from the `Context` and contains vectors of public
//! keys for peers that changed status, categorized by the type of change
//! ("now lost", "now missing", "was lost", "was missing").

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
    ///
    /// This is indicative of a VPN restart.
    pub now_missing: Vec<wireguard::PeerKey>,

    /// Public keys of peers that appeared after being missing (not seen at all)
    /// since the last check.
    pub was_missing: Vec<wireguard::PeerKey>,
}

impl KeyDelta {
    /// Creates a new `KeyDelta` with empty vectors for all categories of
    /// peer status changes.
    pub fn new() -> Self {
        Self {
            now_lost: Vec::new(),
            was_lost: Vec::new(),
            now_missing: Vec::new(),
            was_missing: Vec::new(),
        }
    }

    /// Returns `true` if all the key vectors in the `KeyDelta` are empty, indicating
    /// that there are no changes in peer status since the last check.
    /// `false` if not.
    pub fn is_empty(&self) -> bool {
        self.now_lost.is_empty()
            && self.was_lost.is_empty()
            && self.now_missing.is_empty()
            && self.was_missing.is_empty()
    }

    /// Prints the non-empty key vectors in the `KeyDelta` with a specified prefix
    /// for each line, useful for debugging or logging the changes in peer status.
    ///
    /// # Parameters
    /// - `prefix`: A string prefix to prepend to each line of output, which can
    ///   help visually distinguish this output in terminal output.
    pub fn print_nonempty_keys_prefixed(&self, prefix: &str) {
        print_nonempty_vec_prefixed(prefix, "now lost", &self.now_lost);
        print_nonempty_vec_prefixed(prefix, "was lost", &self.was_lost);
        print_nonempty_vec_prefixed(prefix, "now missing", &self.now_missing);
        print_nonempty_vec_prefixed(prefix, "was missing", &self.was_missing);
    }

    /// Merges another `KeyDelta` into the current `KeyDelta`, effectively
    /// making this one a union of the two.
    ///
    /// # Parameters
    /// - `other`: The other `KeyDelta` to merge into the current one.
    pub fn merge(&mut self, other: &Self) {
        let now_lost_unique_to_other =
            utils::get_elements_not_in_other_vec(&other.now_lost, &self.now_lost);
        let was_lost_unique_to_other =
            utils::get_elements_not_in_other_vec(&other.was_lost, &self.was_lost);
        let now_missing_unique_to_other =
            utils::get_elements_not_in_other_vec(&other.now_missing, &self.now_missing);
        let was_missing_unique_to_other =
            utils::get_elements_not_in_other_vec(&other.was_missing, &self.was_missing);
        self.now_lost.extend(now_lost_unique_to_other);
        self.was_lost.extend(was_lost_unique_to_other);
        self.now_missing.extend(now_missing_unique_to_other);
        self.was_missing.extend(was_missing_unique_to_other);
    }
}

/// Small helper function to print a non-empty vector of peer keys with
/// a prefix string and a description prepended to the line.
///
/// If the vector is empty, nothing is printed.
///
/// # Parameters
/// - `prefix`: A string prefix to prepend to the line, to visually distinguish
///   it in terminal output.
/// - `description`: A string description of the vector to include in the output.
/// - `keys`: A slice of `wireguard::PeerKey` representing the public keys of
///   the peers that changed status.
fn print_nonempty_vec_prefixed(prefix: &str, description: &str, keys: &[wireguard::PeerKey]) {
    if keys.is_empty() {
        return;
    }

    println!(
        "{prefix}{description}: {}",
        keys.iter()
            .map(|k| k.as_str())
            .collect::<Vec<&str>>()
            .join(", ")
    );
}
