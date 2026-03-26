//! Types related to paths to resources, resolved at runtime.

use std::path;

/// Struct containing paths to various resources used by the program, such as
/// the configuration directory, configuration file, and peer list file.
///
/// These paths are resolved at runtime based on environment variables,
/// command-line arguments and hardcoded defaults,
/// and are used by the program to locate the necessary files and directories
/// for its operation.
#[derive(Debug, Default)]
pub struct PathBufs {
    /// Path to the configuration directory, which is the base directory where
    /// the configuration file and peer list file are located.
    pub config_dir: path::PathBuf,

    /// Path to the configuration file, which contains the settings for the program.
    pub config_file: path::PathBuf,

    /// Path to the peer list file, which contains the list of peers to monitor
    /// and their associated information.
    pub peer_list: path::PathBuf,
}
