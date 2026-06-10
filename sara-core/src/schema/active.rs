//! Process-wide active [`Schema`] used by domain methods.
//!
//! Domain types ([`crate::model::ItemType`], [`crate::model::RelationshipType`])
//! resolve their metadata against the active schema rather than hard-coded
//! tables. Callers therefore see the active schema's prefixes, display names,
//! parent links and relationship matrix.
//!
//! The active schema is installed once per process at startup (typically by
//! the CLI after loading `Config`). If no schema is installed, the active
//! schema is the built-in default. An installed schema replaces the built-in
//! model entirely: types and relations it does not define do not exist.

use std::sync::OnceLock;

use super::{ItemTypeDef, RelationDef, Schema};

/// Holds the installed schema, if any. Lazily defaults to [`Schema::builtin`].
static ACTIVE: OnceLock<Schema> = OnceLock::new();

/// Returns the active schema, initializing it to the built-in default on first
/// access if no schema has been installed.
///
/// The returned reference has `'static` lifetime so domain methods can keep
/// returning `&'static str` for prefixes and display names.
#[must_use]
pub fn active() -> &'static Schema {
    ACTIVE.get_or_init(Schema::builtin)
}

/// Installs the process-wide active schema.
///
/// Intended to be called once at startup, before any domain method is invoked.
/// Subsequent calls return the supplied schema back as `Err`, mirroring the
/// underlying [`OnceLock::set`] semantics.
///
/// # Errors
///
/// Returns the supplied schema unchanged if another schema is already active.
pub fn install(schema: Schema) -> Result<(), Schema> {
    ACTIVE.set(schema)
}

/// Returns the definition for an item type id in the active schema.
#[must_use]
pub(crate) fn item_type_def(id: &str) -> Option<&'static ItemTypeDef> {
    active().item_type(id)
}

/// Returns the relation definition for a relation id in the active schema.
#[must_use]
pub(crate) fn relation_def(id: &str) -> Option<&'static RelationDef> {
    active().relation(id)
}
