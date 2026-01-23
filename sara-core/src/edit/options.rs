//! Edit options for modifying items.

use crate::model::TraceabilityLinks;

/// Options for editing an item.
#[derive(Debug, Clone, Default)]
pub struct EditOptions {
    /// The item ID to edit.
    pub item_id: String,
    /// New name (if provided).
    pub name: Option<String>,
    /// New description (if provided).
    pub description: Option<String>,
    /// New refines references (if provided).
    pub refines: Option<Vec<String>>,
    /// New derives_from references (if provided).
    pub derives_from: Option<Vec<String>>,
    /// New satisfies references (if provided).
    pub satisfies: Option<Vec<String>>,
    /// New depends_on references (if provided).
    pub depends_on: Option<Vec<String>>,
    /// New specification (if provided).
    pub specification: Option<String>,
    /// New platform (if provided).
    pub platform: Option<String>,
}

impl EditOptions {
    /// Creates new edit options for the given item ID.
    pub fn new(item_id: impl Into<String>) -> Self {
        Self {
            item_id: item_id.into(),
            ..Default::default()
        }
    }

    /// Sets the name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the name if provided.
    pub fn maybe_name(mut self, name: Option<String>) -> Self {
        self.name = name;
        self
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the description if provided.
    pub fn maybe_description(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }

    /// Sets the refines references.
    pub fn with_refines(mut self, refines: Vec<String>) -> Self {
        self.refines = Some(refines);
        self
    }

    /// Sets the refines references if provided.
    pub fn maybe_refines(mut self, refines: Option<Vec<String>>) -> Self {
        self.refines = refines;
        self
    }

    /// Sets the derives_from references.
    pub fn with_derives_from(mut self, derives_from: Vec<String>) -> Self {
        self.derives_from = Some(derives_from);
        self
    }

    /// Sets the derives_from references if provided.
    pub fn maybe_derives_from(mut self, derives_from: Option<Vec<String>>) -> Self {
        self.derives_from = derives_from;
        self
    }

    /// Sets the satisfies references.
    pub fn with_satisfies(mut self, satisfies: Vec<String>) -> Self {
        self.satisfies = Some(satisfies);
        self
    }

    /// Sets the satisfies references if provided.
    pub fn maybe_satisfies(mut self, satisfies: Option<Vec<String>>) -> Self {
        self.satisfies = satisfies;
        self
    }

    /// Sets the depends_on references.
    pub fn with_depends_on(mut self, depends_on: Vec<String>) -> Self {
        self.depends_on = Some(depends_on);
        self
    }

    /// Sets the depends_on references if provided.
    pub fn maybe_depends_on(mut self, depends_on: Option<Vec<String>>) -> Self {
        self.depends_on = depends_on;
        self
    }

    /// Sets the specification.
    pub fn with_specification(mut self, specification: impl Into<String>) -> Self {
        self.specification = Some(specification.into());
        self
    }

    /// Sets the specification if provided.
    pub fn maybe_specification(mut self, specification: Option<String>) -> Self {
        self.specification = specification;
        self
    }

    /// Sets the platform.
    pub fn with_platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = Some(platform.into());
        self
    }

    /// Sets the platform if provided.
    pub fn maybe_platform(mut self, platform: Option<String>) -> Self {
        self.platform = platform;
        self
    }

    /// Returns true if any modification was requested.
    pub fn has_updates(&self) -> bool {
        self.name.is_some()
            || self.description.is_some()
            || self.refines.is_some()
            || self.derives_from.is_some()
            || self.satisfies.is_some()
            || self.depends_on.is_some()
            || self.specification.is_some()
            || self.platform.is_some()
    }
}

/// Values to apply during editing.
#[derive(Debug, Clone)]
pub struct EditedValues {
    /// The name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Optional specification.
    pub specification: Option<String>,
    /// Optional platform.
    pub platform: Option<String>,
    /// Traceability links.
    pub traceability: TraceabilityLinks,
}

impl EditedValues {
    /// Creates new edited values.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            specification: None,
            platform: None,
            traceability: TraceabilityLinks::default(),
        }
    }

    /// Sets the description.
    pub fn with_description(mut self, description: Option<String>) -> Self {
        self.description = description;
        self
    }

    /// Sets the specification.
    pub fn with_specification(mut self, specification: Option<String>) -> Self {
        self.specification = specification;
        self
    }

    /// Sets the platform.
    pub fn with_platform(mut self, platform: Option<String>) -> Self {
        self.platform = platform;
        self
    }

    /// Sets the traceability links.
    pub fn with_traceability(mut self, traceability: TraceabilityLinks) -> Self {
        self.traceability = traceability;
        self
    }
}
