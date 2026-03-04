//! FIXME

use std::env;
use std::path;

use crate::defaults;

/// Resolves the configuration directory path, returning the directory as a string and an optional PathBuf.
pub fn resolve_default_config_directory_from_env() -> Result<path::PathBuf, String> {
    if let Some(path) = env::var_os("WG_MONITOR_CONFIG_DIR").map(path::PathBuf::from) {
        return Ok(path);
    }

    if users::get_current_uid() == 0 {
        return Ok(path::PathBuf::from("/etc").join(defaults::program_metadata::PROGRAM_ARG0));
    }

    if let Some(path) = env::var_os("XDG_CONFIG_HOME").map(path::PathBuf::from) {
        return Ok(path.join(defaults::program_metadata::PROGRAM_ARG0));
    }

    if let Some(path) = env::var_os("HOME").map(path::PathBuf::from) {
        return Ok(path
            .join(".config")
            .join(defaults::program_metadata::PROGRAM_ARG0));
    }

    Err("could not resolve default configuration directory from environment variables".to_string())
}
