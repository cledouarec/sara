//! Configuration settings structures.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::ConfigError;

/// Main configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Repository configuration.
    #[serde(default)]
    pub repositories: RepositoryConfig,

    /// Validation settings.
    #[serde(default)]
    pub validation: ValidationConfig,

    /// Output settings.
    #[serde(default)]
    pub output: OutputConfig,

    /// Custom templates configuration.
    #[serde(default)]
    pub templates: TemplatesConfig,
}

impl Config {
    /// Creates a new config with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a repository path.
    pub fn add_repository(&mut self, path: impl Into<PathBuf>) {
        self.repositories.paths.push(path.into());
    }

    /// Expands all glob patterns in template paths.
    pub fn expand_template_paths(&self) -> Result<Vec<PathBuf>, ConfigError> {
        let mut result = Vec::new();

        for pattern in &self.templates.paths {
            match glob::glob(pattern) {
                Ok(paths) => {
                    for entry in paths {
                        match entry {
                            Ok(path) => result.push(path),
                            Err(e) => {
                                return Err(ConfigError::InvalidGlobPattern {
                                    pattern: pattern.clone(),
                                    reason: e.to_string(),
                                });
                            }
                        }
                    }
                }
                Err(e) => {
                    return Err(ConfigError::InvalidGlobPattern {
                        pattern: pattern.clone(),
                        reason: e.to_string(),
                    });
                }
            }
        }

        Ok(result)
    }
}

/// Repository configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepositoryConfig {
    /// List of repository paths to scan.
    #[serde(default)]
    pub paths: Vec<PathBuf>,
}

/// Validation settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValidationConfig {
    /// Treat orphan items as errors (true) or warnings (false).
    #[serde(default)]
    pub strict_orphans: bool,

    /// List of allowed custom fields in frontmatter.
    #[serde(default)]
    pub allowed_custom_fields: Vec<String>,
}

/// Output settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Enable colored output.
    #[serde(default = "default_true")]
    pub colors: bool,

    /// Enable emoji output.
    #[serde(default = "default_true")]
    pub emojis: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            colors: true,
            emojis: true,
        }
    }
}

fn default_true() -> bool {
    true
}

/// Custom templates configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemplatesConfig {
    /// Paths to custom template Markdown files (supports glob patterns, e.g., "*.md").
    /// Each template must contain exactly one 'type' field in its YAML frontmatter
    /// to identify the item type it defines.
    /// Custom templates override built-in templates for the corresponding item type.
    #[serde(default)]
    pub paths: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.repositories.paths.is_empty());
        assert!(!config.validation.strict_orphans);
        assert!(config.output.colors);
        assert!(config.output.emojis);
    }

    #[test]
    fn test_add_repository() {
        let mut config = Config::default();
        config.add_repository("/path/to/repo");
        assert_eq!(config.repositories.paths.len(), 1);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("[repositories]"));
    }
}
