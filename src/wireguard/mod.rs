//! WireGuard-related functionality, including parsing peer information and handshakes.

mod peer;
mod wg;

pub use peer::{PeerKey, WireGuardPeer, sort_keys};
pub use wg::{get_handshakes, read_peer_list, update_handshakes, validate_handshakes};
