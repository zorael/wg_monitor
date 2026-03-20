//! This module contains functions related to resolving the configuration directory
//! path from environment variables and default locations.

use std::env;
use std::path;

use crate::defaults;

/// Resolves the default configuration directory for the program based on environment
/// variables and default locations.
///
/// # Notes
/// 1. If the `WG_MONITOR_CONFIG_DIR` environment variable is set, use that
///    as the configuration directory.
/// 2. If the program is running as root (UID 0), use `/etc/<program_name>`
///    as the configuration directory.
/// 3. If the `XDG_CONFIG_HOME` environment variable is set, use
///    `<XDG_CONFIG_HOME>/<program_name>` as the configuration directory.
/// 4. If the `HOME` environment variable is set, use `<HOME>/.config/<program_name>`
///    as the configuration directory.
/// 5. If none of the above conditions are met, return an error indicating that
///    the configuration directory could not be resolved.
///
/// # Returns
/// - `Ok(path)` if the configuration directory was successfully resolved.
/// - `Err(())` if the configuration directory could not be resolved.
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
