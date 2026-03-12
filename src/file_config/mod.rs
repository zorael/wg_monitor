//! Module for handling the file-based configuration of the application.
//! This includes structures that mirror the runtime settings used by the
//! program, but are designed to be deserialized from a configuration file on disk.

mod io;
mod paths;
mod types;

pub use io::deserialize_config_file;
pub use paths::resolve_default_config_directory_from_env;
pub use types::*;
