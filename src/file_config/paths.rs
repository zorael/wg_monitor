//! Module for handling the file-based configuration of the application.

use std::env;
use std::path;

use crate::defaults;

/// Resolves the configuration directory path, returning the directory as a
/// `Result<PathBuf, ()>`, where the unit type `()` indicates an error if the
/// directory could not be resolved.
pub fn resolve_default_config_directory_from_env() -> Result<path::PathBuf, ()> {
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

    Err(())
}
