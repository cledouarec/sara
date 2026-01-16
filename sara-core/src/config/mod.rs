//! Configuration handling for sara.

mod settings;

pub use settings::{
    Config, OutputConfig, RepositoryConfig, TemplatesConfig, ValidationConfig, expand_glob_patterns,
};

use std::path::Path;

use crate::error::ConfigError;

/// Default configuration file name.
pub const DEFAULT_CONFIG_FILE: &str = "sara.toml";

/// Loads configuration from a TOML file.
pub fn load_config(path: &Path) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path).map_err(|e| ConfigError::FileRead {
        path: path.to_path_buf(),
        reason: e.to_string(),
    })?;

    parse_config(&content, path)
}

/// Parses configuration from a TOML string.
pub fn parse_config(content: &str, path: &Path) -> Result<Config, ConfigError> {
    toml::from_str(content).map_err(|e| ConfigError::InvalidConfig {
        path: path.to_path_buf(),
        reason: e.to_string(),
    })
}

/// Loads configuration from the default location or returns default config.
pub fn load_or_default(path: Option<&Path>) -> Result<Config, ConfigError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_config() {
        let toml = r#"
[repositories]
paths = ["./docs", "../other-repo"]

[validation]
strict_orphans = true

[output]
colors = true
emojis = false

[templates]
paths = ["./templates/*.md"]
"#;

        let config = parse_config(toml, Path::new("test.toml")).unwrap();
        assert_eq!(config.repositories.paths.len(), 2);
        assert!(config.validation.strict_orphans);
        assert!(config.output.colors);
        assert!(!config.output.emojis);
        assert_eq!(config.templates.paths.len(), 1);
    }

    #[test]
    fn test_parse_minimal_config() {
        let toml = r#"
[repositories]
paths = ["./docs"]
"#;

        let config = parse_config(toml, Path::new("test.toml")).unwrap();
        assert_eq!(config.repositories.paths, vec![PathBuf::from("./docs")]);
        assert!(!config.validation.strict_orphans);
        assert!(config.output.colors);
        assert!(config.output.emojis);
    }

    #[test]
    fn test_parse_empty_config() {
        let config = parse_config("", Path::new("test.toml")).unwrap();
        assert!(config.repositories.paths.is_empty());
    }
}
