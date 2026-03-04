//! Struct aggregating PathBufs used in the program.

use std::path::PathBuf;

/// Paths to resources, resolved at runtime.
#[derive(Debug, Default)]
pub struct PathBufs {
    /// Path to the configuration directory, which contains the configuration file and other resources.
    pub config_dir: PathBuf,

    /// Path to the configuration file.
    pub config_file: PathBuf,

    /// Path to the peer list file.
    pub peer_list: PathBuf,
}
