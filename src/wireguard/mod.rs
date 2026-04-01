//! WireGuard-related functionality.

mod io;
mod peer;
mod wg;

pub use io::{parse_peer_list, read_peer_list_file};
pub use peer::{PeerKey, WireGuardPeer, sort_keys};
pub use wg::{get_handshakes, resolve_wg, update_handshakes, validate_handshakes};
