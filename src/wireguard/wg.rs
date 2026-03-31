//! WireGuard-related functions for reading peer information and handshakes,
//! specifically in the context of the output of the `wg` command.

use std::collections;
use std::env;
use std::fs;
use std::io;
use std::io::BufRead;
use std::path;
use std::process;

use crate::defaults;

/// Reads a list of WireGuard peers from a specified file path, returning a
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
/// - `path`: The file path to read the peers from.
/// - `debug`: A boolean flag indicating whether to print debug information
///   during the reading process.
///
/// # Returns
/// An `io::Result` containing a `collections::HashMap` of `PeerKey` keys to
/// `WireGuardPeer` values if successful, or an `io::Error` if there was an
/// issue reading the file or parsing its contents.
pub fn read_peer_list(
    path: &path::Path,
    debug: bool,
) -> io::Result<collections::HashMap<super::PeerKey, super::WireGuardPeer>> {
    if debug {
        println!("Reading peers from file: '{}'\n", path.display());
    }

    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let mut peers = collections::HashMap::new();

    for whole_line in reader.lines() {
        let whole_line = whole_line?;
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
                eprintln!(
                    "Invalid line in peers file (missing public key): '{}'",
                    whole_line
                );
                continue;
            }
        };

        let human_name = parts.next().map(str::trim_start);

        let Some(peer) = super::WireGuardPeer::new(key, human_name) else {
            eprintln!("Invalid public key in peers file: '{}'", key);
            continue;
        };

        if debug {
            println!("{:#?}\n", peer);
        }

        match peers.entry(peer.public_key.clone()) {
            collections::hash_map::Entry::Vacant(e) => e.insert(peer),
            collections::hash_map::Entry::Occupied(_) => {
                eprintln!(
                    "Duplicate public key in peers file: '{}'. Skipping.",
                    peer.public_key
                );
                continue;
            }
        };
    }

    if debug {
        println!("Total peers loaded: {}\n", peers.len());
    }

    Ok(peers)
}

/// "Validates" the output of the `wg show {iface} latest-handshakes` command,
/// ensuring that each line contains what seems to be a public key and a timestamp.
///
/// The function only checks that each line is in the format "`public_key\ttimestamp`".
/// It does *not* do a real cryptographic validation of the keys.
///
/// # Parameters
/// - `terminal_output`: The output from `wg show {iface} latest-handshakes`.
///
/// # Returns
/// - `Ok(())` if all lines are valid.
/// - `Err(Vec<String>)` if there are invalid lines, containing a vector of
///   descriptive error messages detailing each issue found.
pub fn validate_handshakes(terminal_output: &str) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    for line in terminal_output.lines() {
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        let Some((key, timestamp)) = line.split_once('\t') else {
            errors.push(format!("Invalid line in handshakes output: '{line}'"));
            continue;
        };

        if timestamp.parse::<u64>().is_err() {
            errors.push(format!("Invalid timestamp for key '{key}': '{timestamp}'"));
            continue;
        };
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Updates the last seen timestamps for peers based on the output of the
/// `wg show {iface} latest-handshakes` command.
///
/// The function first resets the last seen timestamps for all peers, then
/// parses the command output line by line. For each valid line, it updates the
/// corresponding peer's last seen timestamp based on the provided UNIX timestamp.
///
/// If a peer is not present in the command output, its last seen timestamp
/// will stay reset (`None` and `0`). If it was previously present, this means the
/// peer has been removed from the VPN, and the current behavior is to treat
/// it as having gone missing.
///
/// # Parameters
/// - `terminal_output`: The output from `wg show {iface} latest-handshakes`,
///   which should contain lines in the format "`public_key\ttimestamp`".
/// - `peers`: A mutable reference to a `collections::HashMap` of `PeerKey` keys to
///   `WireGuardPeer` values, to be updated based on the command output.
pub fn update_handshakes(
    terminal_output: &str,
    peers: &mut collections::HashMap<super::PeerKey, super::WireGuardPeer>,
) {
    for peer in peers.values_mut() {
        // Reset all peers prior to updating, so that any peers not present
        // in the command output will be marked as missing (last seen None, unix 0).
        // This should only happen when a peer is removed from the VPN.
        peer.reset_last_seen();
    }

    for line in terminal_output.lines() {
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        let Some((key, timestamp)) = line.split_once('\t') else {
            continue;
        };

        // Borrow magic to avoid creating a new PeerKey instance just to look it up
        let Some(peer) = peers.get_mut(key) else {
            continue;
        };

        match timestamp.parse::<u64>() {
            Ok(0) | Err(_) => peer.reset_last_seen(),
            Ok(seconds) => peer.set_last_seen(seconds),
        };
    }
}

/// Executes the `wg show {iface} latest-handshakes` command for the specified
/// interface and returns its output as a `String`.
///
/// The `LC_ALL` environment variable is set to "`C`" to ensure that the output
/// format is consistent across locales.
///
/// # Parameters
/// - `wg`: The path to the `wg` executable to use for running the command.
/// - `interface`: The name of the WireGuard interface to query (like "`wg0`").
///
/// # Returns
/// An `io::Result` containing the command output as a `String` if successful, or an
/// `io::Error` if there was an issue executing the command, or if the command
/// returned a non-zero exit status.
pub fn get_handshakes(wg: &path::PathBuf, interface: &str) -> io::Result<String> {
    let output = process::Command::new(wg)
        .arg("show")
        .arg(interface)
        .arg("latest-handshakes")
        .env("LC_ALL", "C") // Ensure consistent output format regardless of locale
        .output()?;

    if !output.status.success() {
        return Err(io::Error::other(
            String::from_utf8_lossy(&output.stderr).trim(),
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Resolves the path to the `wg` executable, checking environment variables and
/// default locations.
///
/// The function checks the following in order:
/// 1. If the `WG_MONITOR_WG_PATH` environment variable is set,
///    it uses that as the path to the `wg` executable.
/// 2. If the default path defined in `defaults::WG_PATH` exists, it uses that.
/// 3. If neither of the above conditions are met, it falls back to just
///    "`wg`", relying on the system's `$PATH` to find it.
///
/// # Returns
/// A `path::PathBuf` representing the hopefully-resolved path to the `wg` executable.
pub fn resolve_wg() -> path::PathBuf {
    if let Some(from_env) = env::var_os("WG_MONITOR_WG_PATH").map(path::PathBuf::from) {
        return from_env;
    }

    let from_defaults = path::PathBuf::from(defaults::WG_PATH);

    if from_defaults.exists() {
        return from_defaults;
    }

    path::PathBuf::from("wg")
}
