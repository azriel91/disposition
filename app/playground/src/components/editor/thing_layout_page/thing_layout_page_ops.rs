//! Thing layout page-level mutation helpers.
//!
//! Provides [`ThingLayoutPageOps`] which groups add/remove operations for the
//! Thing Layout editor page so that related functions are discoverable when
//! sorted by name, per the project's `noun_verb` naming convention.

use dioxus::signals::{ReadableExt, Signal, WritableExt};
use disposition::{
    input_model::{
        thing::{ThingHierarchy, ThingId},
        InputDiagram,
    },
    model_common::Id,
};

/// Page-level mutation operations for the Thing Layout editor page.
///
/// These handle adding new entries to and removing entries from the
/// [`ThingHierarchy`].
pub struct ThingLayoutPageOps;

impl ThingLayoutPageOps {
    /// Adds a new thing to the top level of the hierarchy with a unique
    /// placeholder ID.
    ///
    /// The placeholder follows the pattern `"thing_N"` where `N` is chosen
    /// so that no existing key in the hierarchy (at any depth) collides.
    pub fn entry_add(mut input_diagram: Signal<InputDiagram<'static>>) {
        let mut n = 0usize;
        loop {
            let candidate = format!("thing_{n}");
            if let Ok(id) = Id::new(&candidate) {
                let thing_id = ThingId::from(id.into_static());
                if !Self::hierarchy_contains_recursive(
                    &input_diagram.read().thing_hierarchy,
                    &thing_id,
                ) {
                    input_diagram
                        .write()
                        .thing_hierarchy
                        .insert(thing_id, ThingHierarchy::new());
                    break;
                }
            }
            n += 1;
        }
    }

    /// Removes a thing (and its subtree) from the hierarchy by its string
    /// ID.
    ///
    /// Searches recursively through all nesting levels. The first matching
    /// key is removed along with all of its descendants.
    pub fn entry_remove(mut input_diagram: Signal<InputDiagram<'static>>, thing_id_str: &str) {
        if let Ok(id) = Id::new(thing_id_str) {
            let thing_id = ThingId::from(id.into_static());
            Self::hierarchy_remove_recursive(&mut input_diagram.write().thing_hierarchy, &thing_id);
        }
    }

    /// Returns `true` if the hierarchy contains the given `ThingId` at any
    /// nesting level.
    fn hierarchy_contains_recursive<'id>(
        hierarchy: &ThingHierarchy<'id>,
        thing_id: &ThingId<'id>,
    ) -> bool {
        if hierarchy.contains_key(thing_id) {
            return true;
        }
        hierarchy
            .values()
            .any(|child| Self::hierarchy_contains_recursive(child, thing_id))
    }

    /// Recursively removes a `ThingId` key (and its subtree) from a
    /// [`ThingHierarchy`].
    ///
    /// Returns `true` if the key was found and removed.
    fn hierarchy_remove_recursive(
        hierarchy: &mut ThingHierarchy<'static>,
        thing_id: &ThingId<'static>,
    ) -> bool {
        if hierarchy.swap_remove(thing_id).is_some() {
            return true;
        }
        hierarchy
            .values_mut()
            .any(|child| Self::hierarchy_remove_recursive(child, thing_id))
    }
}
