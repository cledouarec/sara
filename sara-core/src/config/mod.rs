//! Configuration handling for sara.

mod settings;

pub use settings::{Config, OutputConfig, RepositoryConfig, TemplatesConfig, ValidationConfig};

use std::path::Path;

use crate::error::SaraError;

/// Default configuration file name.
pub const DEFAULT_CONFIG_FILE: &str = "sara.toml";

/// Loads configuration from a TOML file.
pub fn load_config(path: &Path) -> Result<Config, SaraError> {
    let content = std::fs::read_to_string(path).map_err(|e| SaraError::ConfigRead {
        path: path.to_path_buf(),
        reason: e.to_string(),
    })?;

    toml::from_str(&content).map_err(|e| SaraError::InvalidConfig {
        path: path.to_path_buf(),
        reason: e.to_string(),
    })
}

/// Loads configuration from the default location or returns default config.
pub fn load_or_default(path: Option<&Path>) -> Result<Config, SaraError> {
    match path {
        Some(p) => load_config(p),
        None => {
            let default_path = Path::new(DEFAULT_CONFIG_FILE);
            if default_path.exists() {
                load_config(default_path)
            } else {
                Ok(Config::default())
            }
        }
    }
}
