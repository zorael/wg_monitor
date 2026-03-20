//! This module contains functions and types related to the configuration file
//! used by the program.

mod io;
mod paths;
mod types;

pub use io::deserialize_config_file;
pub use paths::resolve_default_config_directory_from_env;
pub use types::*;
