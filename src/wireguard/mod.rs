//! WireGuard-related functionality.

mod peer;
mod wg;

pub use peer::{PeerKey, WireGuardPeer, sort_keys};
pub use wg::{get_handshakes, read_peer_list, resolve_wg, update_handshakes, validate_handshakes};
