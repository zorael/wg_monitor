//! WireGuard-related functionality.

mod peer;
mod wg;

pub use peer::{PeerKey, WireGuardPeer, sort_keys};
pub use wg::{
    get_handshakes, parse_peer_list, read_peer_list_file, resolve_wg, update_handshakes,
    validate_handshakes,
};
