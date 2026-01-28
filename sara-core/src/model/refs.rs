//! Reference types for upstream and downstream relationships.

use serde::{Deserialize, Serialize};

use super::ItemId;

/// Upstream relationship references (this item points to parents).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpstreamRefs {
    /// Items this item refines (for UseCase, Scenario).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refines: Vec<ItemId>,

    /// Items this item derives from (for SystemRequirement, HW/SW Requirement).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub derives_from: Vec<ItemId>,

    /// Items this item satisfies (for SystemArchitecture, HW/SW DetailedDesign).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub satisfies: Vec<ItemId>,

    /// Design artifacts this ADR justifies (for ArchitectureDecisionRecord).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub justifies: Vec<ItemId>,
}

impl UpstreamRefs {
    /// Returns an iterator over all upstream item IDs.
    pub fn all_ids(&self) -> impl Iterator<Item = &ItemId> {
        self.refines
            .iter()
            .chain(self.derives_from.iter())
            .chain(self.satisfies.iter())
            .chain(self.justifies.iter())
    }

    /// Returns true if there are no upstream references.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.refines.is_empty()
            && self.derives_from.is_empty()
            && self.satisfies.is_empty()
            && self.justifies.is_empty()
    }
}

/// Downstream relationship references (this item points to children).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DownstreamRefs {
    /// Items that refine this item (for Solution, UseCase).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub is_refined_by: Vec<ItemId>,

    /// Items derived from this item (for Scenario, SystemArchitecture).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub derives: Vec<ItemId>,

    /// Items that satisfy this item (for SystemRequirement, HW/SW Requirement).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub is_satisfied_by: Vec<ItemId>,

    /// ADRs that justify this item (for design artifacts: SYSARCH, SWDD, HWDD).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub justified_by: Vec<ItemId>,
}

impl DownstreamRefs {
    /// Returns an iterator over all downstream item IDs.
    pub fn all_ids(&self) -> impl Iterator<Item = &ItemId> {
        self.is_refined_by
            .iter()
            .chain(self.derives.iter())
            .chain(self.is_satisfied_by.iter())
            .chain(self.justified_by.iter())
    }

    /// Returns true if there are no downstream references.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.is_refined_by.is_empty()
            && self.derives.is_empty()
            && self.is_satisfied_by.is_empty()
            && self.justified_by.is_empty()
    }
}
