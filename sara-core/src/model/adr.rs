//! ADR (Architecture Decision Record) lifecycle types.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Represents the lifecycle status of an ADR.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdrStatus {
    /// Decision is under consideration, not yet finalized.
    Proposed,
    /// Decision has been approved and is in effect.
    Accepted,
    /// Decision is no longer recommended but not replaced.
    Deprecated,
    /// Decision has been replaced by a newer ADR.
    Superseded,
}

impl AdrStatus {
    /// Returns the display name for this status.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Proposed => "Proposed",
            Self::Accepted => "Accepted",
            Self::Deprecated => "Deprecated",
            Self::Superseded => "Superseded",
        }
    }

    /// Returns the YAML value (snake_case string) for this status.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Accepted => "accepted",
            Self::Deprecated => "deprecated",
            Self::Superseded => "superseded",
        }
    }
}

impl fmt::Display for AdrStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl FromStr for AdrStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "proposed" => Ok(Self::Proposed),
            "accepted" => Ok(Self::Accepted),
            "deprecated" => Ok(Self::Deprecated),
            "superseded" => Ok(Self::Superseded),
            _ => Err(format!(
                "Invalid ADR status '{s}'. Expected one of: proposed, accepted, deprecated, superseded"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adr_status_display() {
        assert_eq!(AdrStatus::Proposed.to_string(), "Proposed");
        assert_eq!(AdrStatus::Accepted.to_string(), "Accepted");
        assert_eq!(AdrStatus::Deprecated.to_string(), "Deprecated");
        assert_eq!(AdrStatus::Superseded.to_string(), "Superseded");
    }

    #[test]
    fn test_adr_status_from_str() {
        assert_eq!(
            "proposed".parse::<AdrStatus>().unwrap(),
            AdrStatus::Proposed
        );
        assert_eq!(
            "ACCEPTED".parse::<AdrStatus>().unwrap(),
            AdrStatus::Accepted
        );
        assert!("invalid".parse::<AdrStatus>().is_err());
    }

    #[test]
    fn test_adr_status_as_str() {
        assert_eq!(AdrStatus::Proposed.as_str(), "proposed");
        assert_eq!(AdrStatus::Superseded.as_str(), "superseded");
    }
}
