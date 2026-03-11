//! Struct aggregating PathBufs used in the program.

use std::path;

/// Paths to resources, resolved at runtime.
#[derive(Debug, Default)]
pub struct PathBufs {
    /// Path to the configuration directory, which contains the configuration
    /// file and other resources.
    pub config_dir: path::PathBuf,

    /// Path to the configuration file.
    pub config_file: path::PathBuf,

    /// Path to the peer list file.
    pub peer_list: path::PathBuf,
}
