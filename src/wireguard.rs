//! Wireguard-related functionality for monitoring peers and handshakes.
//!
//! This module provides functions to read the list of Wireguard peers from a file,
//! validate the output of the `wg show latest-handshakes` command, update the
//! last seen timestamps for peers based on the command output, and get the
//! terminal output from running the command.

use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path;
use std::time;

use crate::peer;

/// Reads the list of Wireguard peers from a file, returning a HashMap of
/// public keys to `WireguardPeer` structs.
pub fn read_peer_list(
    path: &path::Path,
    debug: bool,
) -> io::Result<HashMap<String, peer::WireguardPeer>> {
    if debug {
        println!("Reading peers from file: '{}'\n", path.display());
    }

    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut peers = HashMap::new();

    for line in reader.lines() {
        let line = line?.trim().to_string();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if debug {
            println!("{line}");
        }

        if let Some((key, human_name)) = line.split_once(' ') {
            if !peer::WireguardPeer::validate_public_key(key) {
                eprintln!("Warning: Invalid public key in peers file: '{}'", key);
                continue;
            }

            let peer = peer::WireguardPeer {
                public_key: key.to_string(),
                human_name: human_name.to_string(),
                last_seen: None,
                timestamp: None,
            };

            if debug {
                println!("{:#?}\n", peer);
            }

            peers.insert(key.to_string(), peer);
        } else if peer::WireguardPeer::validate_public_key(&line) {
            let key = line.to_string();
            let peer = peer::WireguardPeer {
                public_key: key.clone(),
                human_name: peer::WireguardPeer::shorten_key(&key),
                last_seen: None,
                timestamp: None,
            };

            if debug {
                println!("{:#?}\n", peer);
            }

            peers.insert(key, peer);
        } else {
            // Invalid line format, skip it
            eprintln!("Warning: Invalid line in peers file: '{}'", line);
        }
    }

    if debug {
        println!("Total peers loaded: {}\n", peers.len());
    }

    Ok(peers)
}

/// Validates the list of peers, returning a vector of error messages for any obvious issues found.
pub fn validate_handshakes(terminal_output: &str) -> Vec<String> {
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

        if timestamp.parse::<u64>().ok().is_none() {
            errors.push(format!("Invalid timestamp for key '{key}': '{timestamp}'"));
            continue;
        };
    }

    errors
}

/// Updates the last seen timestamps for peers based on the output of the
/// `wg show iface latest-handshakes` command.
pub fn update_handshakes(terminal_output: &str, peers: &mut HashMap<String, peer::WireguardPeer>) {
    for line in terminal_output.lines() {
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        let Some((key, timestamp)) = line.split_once('\t') else {
            continue;
        };

        let Some(peer) = peers.get_mut(key) else {
            continue;
        };

        match timestamp.parse::<u64>() {
            Ok(0) | Err(_) => {
                peer.timestamp = None;
                peer.last_seen = None;
            }
            Ok(seconds) => {
                peer.timestamp = Some(seconds);
                peer.last_seen = Some(time::UNIX_EPOCH + time::Duration::from_secs(seconds));
            }
        };
    }
}

/// Gets the terminal output from running the `wg show iface latest-handshakes` command,
/// returning an error if the command fails or produces invalid output.
pub fn get_handshakes(interface: &str) -> io::Result<String> {
    let output = std::process::Command::new("/usr/bin/wg")
        .arg("show")
        .arg(interface)
        .arg("latest-handshakes")
        .output()?;

    if !output.status.success() {
        println!("{}", String::from_utf8_lossy(&output.stderr));

        return Err(io::Error::other(format!(
            "Command 'wg show {interface} latest-handshakes' failed with status: {}",
            output.status
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
