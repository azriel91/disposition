//! Categories of document-defined IDs, mapped from schema `$ref` names.

/// A category of ID that a value position may reference, derived from the schema
/// type of the value (its `$ref` name).
///
/// For example, an `EdgeGroup`'s `things` array has items `{ "$ref":
/// "#/$defs/ThingId" }`, so completing one of its values should offer the
/// [`IdCategory::Thing`] IDs already defined in the document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdCategory {
    /// A `ThingId` -- defined under `things` / `thing_hierarchy`.
    Thing,
    /// A `TagId` -- defined under `tags`.
    Tag,
    /// A `ProcessStepId` -- defined under a process's `steps`.
    ProcessStep,
    /// An `EdgeGroupId` -- defined under `thing_dependencies` /
    /// `thing_interactions`.
    EdgeGroup,
    /// Any ID -- the generic `Id` type (e.g. `thing_layouts` keys).
    Any,
}

impl IdCategory {
    /// Maps a schema `$defs` name to the ID category it references, if any.
    pub fn from_ref_name(ref_name: &str) -> Option<IdCategory> {
        match ref_name {
            "ThingId" => Some(IdCategory::Thing),
            "TagId" => Some(IdCategory::Tag),
            "ProcessStepId" => Some(IdCategory::ProcessStep),
            "EdgeGroupId" => Some(IdCategory::EdgeGroup),
            "Id" => Some(IdCategory::Any),
            _ => None,
        }
    }
}
