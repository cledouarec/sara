//! Process-wide active [`Schema`] used by domain methods.
//!
//! Domain enums ([`crate::model::ItemType`], [`crate::model::RelationshipRules`])
//! delegate their metadata lookups to the active schema rather than hard-coded
//! tables. Callers therefore see the active schema's prefixes, display names,
//! parent links and relationship matrix.
//!
//! The active schema is installed once per process at startup (typically by the
//! CLI after loading `Config`). If no schema is installed, the active schema is
//! the built-in default, preserving the legacy behavior exactly.

use std::sync::OnceLock;

use super::{ItemTypeDef, RelationDef, Schema};

/// Holds the installed schema, if any. Lazily defaults to [`Schema::builtin`].
static ACTIVE: OnceLock<Schema> = OnceLock::new();

/// Cached built-in schema, used as a fallback when the active schema is
/// missing a definition for a known enum variant.
static BUILTIN: OnceLock<Schema> = OnceLock::new();

/// Returns the active schema, initializing it to the built-in default on first
/// access if no schema has been installed.
///
/// The returned reference has `'static` lifetime so domain methods can keep
/// returning `&'static str` for prefixes and display names.
#[must_use]
pub fn active() -> &'static Schema {
    ACTIVE.get_or_init(Schema::builtin)
}

/// Returns the cached built-in schema, used as a fallback for variants the
/// active schema does not define.
fn builtin() -> &'static Schema {
    BUILTIN.get_or_init(Schema::builtin)
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

/// Returns the definition for an item type id, falling back to the built-in
/// schema if the active schema does not define it.
///
/// Used by [`crate::model::ItemType`] methods so that adding a known enum
/// variant to a custom schema is optional: the variant still works against
/// the built-in defaults.
#[must_use]
pub(crate) fn item_type_def(id: &str) -> Option<&'static ItemTypeDef> {
    active().item_type(id).or_else(|| builtin().item_type(id))
}

/// Returns the relation definition for a relation id, falling back to the
/// built-in schema if the active schema does not define it.
#[must_use]
pub(crate) fn relation_def(id: &str) -> Option<&'static RelationDef> {
    active().relation(id).or_else(|| builtin().relation(id))
}

/// Returns the definitions of all known item types.
///
/// Active-schema types come first in declaration order, followed by built-in
/// types the active schema does not redefine, so that a partial custom schema
/// still exposes the full default model.
#[must_use]
pub(crate) fn all_item_type_defs() -> &'static [&'static ItemTypeDef] {
    static ALL: OnceLock<Vec<&'static ItemTypeDef>> = OnceLock::new();
    ALL.get_or_init(|| {
        let mut defs: Vec<&'static ItemTypeDef> = active().item_types.iter().collect();
        defs.extend(
            builtin()
                .item_types
                .iter()
                .filter(|def| active().item_type(&def.id).is_none()),
        );
        defs
    })
}
