//! ADR (Architecture Decision Record) types.

use serde::{Deserialize, Serialize};
use std::fmt;

/// ADR lifecycle status.
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

    /// Returns all possible ADR status values.
    #[must_use]
    pub const fn all() -> &'static [AdrStatus] {
        &[
            Self::Proposed,
            Self::Accepted,
            Self::Deprecated,
            Self::Superseded,
        ]
    }
}

impl fmt::Display for AdrStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
