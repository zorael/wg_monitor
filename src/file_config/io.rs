//! FIXME

use std::path;

/// Deserializes the configuration file from disk, returning an optional FileConfig.
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
