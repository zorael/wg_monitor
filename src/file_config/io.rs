//! This module contains functions for reading and writing the configuration file to disk.
//!
//! "Configuration" explicitly only refers to file-based configuration, as opposed
//! to "settings" which refers to the runtime settings used by the program.
//!
//! The configuration file is deserialized into a `FileConfig` struct, which contains
//! the settings for the program. It in turn contains nested structs for the
//! various sections of the configuration.

use std::path;

/// Deserializes the configuration file at the given path into a `FileConfig` struct.
///
/// If the file does not exist, returns `Ok(None)`. If the file exists but
/// cannot be deserialized, returns an error.
///
/// # Parameters
/// - `config_file`: The path to the configuration file to deserialize.
///
/// # Returns
/// - `Ok(Some(FileConfig))` if the file was successfully deserialized.
/// - `Ok(None)` if the file does not exist.
/// - `Err(ConfyError)` if the file exists but could not be deserialized
pub fn deserialize_config_file(
    config_file: &path::Path,
) -> Result<Option<super::FileConfig>, confy::ConfyError> {
    if !config_file.exists() {
        return Ok(None);
    }

    match confy::load_path(config_file) {
        Ok(cfg) => Ok(Some(cfg)),
        Err(e) => Err(e),
    }
}
