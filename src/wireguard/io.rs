//! WireGuard-related functions for reading and parsing the peer list file.

use std::collections;
use std::fs;
use std::io;
use std::path;

/// Reads the contents of a peer list file at the specified path, returning it as a `String`.
///
/// The function reads the entire contents of the file into a string and returns it.
/// If the `debug` flag is set to `true`, it will print a message indicating
/// what file is being read.
///
/// # Parameters
/// - `path`: The file path to read the peer list from.
/// - `debug`: A boolean flag indicating whether to print debug information about the file being read
///
/// # Returns
/// An `io::Result` containing the contents of the file as a `String` if
/// successful, or an `io::Error` if there was an issue reading the file.
pub fn read_peer_list_file(path: &path::Path, debug: bool) -> io::Result<String> {
    if debug {
        println!("Reading peers from file: '{}'\n", path.display());
    }

    let contents = fs::read_to_string(path)?;
    Ok(contents)
}

/// Parses the contents of a peer list file (passed as a `String`), returning a
/// `collections::HashMap` of `PeerKey` keys to `WireGuardPeer` values.
///
/// The function expects the file to contain lines in one of the following formats:
/// - `public_key human_name`: A public key followed by a human-readable name,
///   with the two separated by whitespace.
/// - `public_key`: Just a public key, in which case a human-readable name will
///   be derived from the public key using the `shorten_key` function.
///
/// The function ignores empty lines and lines starting with `#`, which are
/// treated as comments. If the `debug` flag is set to `true`, the function will
/// print debug information about the peers being read and loaded from the file.
///
/// # Parameters
/// - `contents`: The contents of the peer list file as a `String`.
/// - `debug`: A boolean flag indicating whether to print debug information
///   during the parsing process.
///
/// # Returns
/// - `Ok(HashMap<PeerKey, WireGuardPeer>)` if the parsing was successful,
///   containing a hashmap of `PeerKey` keys to `WireGuardPeer` values.
/// - `Err(Vec<String>)` if there were issues during parsing, containing a
///   vector of descriptive error messages for each issue found.
pub fn parse_peer_list(
    contents: &str,
    debug: bool,
) -> Result<collections::HashMap<super::PeerKey, super::WireGuardPeer>, Vec<String>> {
    let mut peers = collections::HashMap::new();
    let mut errors = Vec::new();

    for whole_line in contents.lines() {
        let whole_line = whole_line.trim();

        if whole_line.is_empty() || whole_line.starts_with('#') {
            continue;
        }

        if debug {
            println!("{whole_line}");
        }

        let mut parts = whole_line.splitn(2, char::is_whitespace);

        let key = match parts.next() {
            Some(k) if !k.is_empty() => k,
            _ => {
                errors.push(format!(
                    "Invalid line in peers file (missing public key): '{}'",
                    whole_line
                ));
                continue;
            }
        };

        let human_name = parts.next().map(str::trim_start);

        let Some(peer) = super::WireGuardPeer::new(key, human_name) else {
            errors.push(format!("Malformed public key in peers file: '{}'", key));
            continue;
        };

        if debug {
            println!("{:#?}\n", peer);
        }

        match peers.entry(peer.public_key.clone()) {
            collections::hash_map::Entry::Vacant(e) => e.insert(peer),
            collections::hash_map::Entry::Occupied(_) => {
                errors.push(format!(
                    "Duplicate public key in peers file: '{}'.",
                    peer.public_key
                ));
                continue;
            }
        };
    }

    if errors.is_empty() {
        Ok(peers)
    } else {
        Err(errors)
    }
}
