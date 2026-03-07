//! Things page mutation helpers.
//!
//! Also contains `IdDuplicateParts`, a small helper used by
//! [`ThingsPageOps::thing_duplicate`] to derive unique copy IDs.
//!
//! Provides [`ThingsPageOps`] which groups all mutation operations for the
//! Things editor page so that related functions are discoverable when sorted
//! by name, per the project's `noun_verb` naming convention.
//!
//! All methods operate on `&mut InputDiagram<'static>` instead of a
//! framework-specific signal type, making them testable without a UI runtime.

use disposition_input_model::{
    edge::EdgeGroup,
    theme::{IdOrDefaults, ThemeStyles},
    thing::{ThingHierarchy, ThingId},
    InputDiagram,
};
use disposition_model_common::{Id, Set};

use crate::{
    id_parse::{parse_id, parse_thing_id},
    id_rename::id_rename_in_input_diagram,
    on_change_target::OnChangeTarget,
};

/// Components of a duplicate thing ID used during
/// [`ThingsPageOps::thing_duplicate`].
///
/// Holds the fixed prefix, the first candidate number, and the minimum digit
/// width so that leading zeroes are preserved (e.g. `_001` -> `_002`).
struct IdDuplicateParts {
    /// Fixed prefix including a trailing underscore, e.g. `"t_foo_"` or
    /// `"t_bar_copy_"`.
    prefix: String,
    /// First candidate number to try (already incremented past the
    /// original).
    start_number: u32,
    /// Minimum number of digit characters, for leading-zero preservation.
    ///
    /// For example, `3` means the number `2` is formatted as `"002"`.
    min_width: usize,
}

impl IdDuplicateParts {
    /// Formats a candidate duplicate ID from the stored prefix and the
    /// given number, zero-padded to at least `min_width` digits.
    ///
    /// Examples: prefix `"t_foo_"`, n `2`, min_width `3` produces
    /// `"t_foo_002"`.
    fn format(&self, n: u32) -> String {
        let min_width = self.min_width;
        format!("{}{n:0>min_width$}", self.prefix)
    }

    /// Returns the insertion index that keeps entries sharing this prefix
    /// in numeric order within an ordered map.
    ///
    /// Scans `keys` (an iterator of string representations in map order)
    /// for entries whose string form starts with `self.prefix` and whose
    /// remaining suffix parses as a `u32`. The duplicate is placed just
    /// after the last sibling whose numeric suffix is less than
    /// `dup_number`.
    ///
    /// If no sibling with a smaller number is found, the duplicate is
    /// inserted just after the original at `orig_index`.
    fn sorted_insert_index<'a>(
        &self,
        keys: impl Iterator<Item = (usize, &'a str)>,
        orig_index: usize,
        dup_number: u32,
    ) -> usize {
        let mut best = orig_index;
        for (idx, key_str) in keys {
            if let Some(suffix) = key_str.strip_prefix(&self.prefix) {
                if !suffix.is_empty()
                    && suffix.chars().all(|c| c.is_ascii_digit())
                    && let Ok(n) = suffix.parse::<u32>()
                    && n < dup_number
                    && idx >= best
                {
                    best = idx;
                }
            }
        }
        best + 1
    }
}

/// Mutation operations for the Things editor page.
///
/// Grouped here so that related functions are discoverable when sorted by
/// name, per the project's `noun_verb` naming convention.
pub struct ThingsPageOps;

impl ThingsPageOps {
    // === Thing helpers === //

    /// Adds a new thing row with a unique placeholder ID.
    ///
    /// Also inserts a corresponding entry into `thing_hierarchy` with an
    /// empty hierarchy so the thing appears in the layout editor.
    pub fn thing_add(input_diagram: &mut InputDiagram<'static>) {
        let mut n = input_diagram.things.len();
        loop {
            let candidate = format!("thing_{n}");
            if let Some(thing_id) = parse_thing_id(&candidate)
                && !input_diagram.things.contains_key(&thing_id)
            {
                input_diagram.things.insert(thing_id.clone(), String::new());
                input_diagram
                    .thing_hierarchy
                    .insert(thing_id, ThingHierarchy::new());
                break;
            }
            n += 1;
        }
    }

    /// Duplicates a thing and all its entries across the [`InputDiagram`].
    ///
    /// The duplicate ID is derived from the original:
    ///
    /// * If the original ends with `_<number>` (e.g. `t_something_1`), the
    ///   number is incremented (`t_something_2`). Leading zeroes are preserved
    ///   when the incremented value fits in the same width (e.g.
    ///   `t_something_001` becomes `t_something_002`), but an extra digit is
    ///   added when it overflows (e.g. `_999` becomes `_1000`).
    /// * Otherwise `_copy_1` is appended (e.g. `t_foo` becomes `t_foo_copy_1`).
    ///   A subsequent duplicate of `t_foo_copy_1` will increment the trailing
    ///   number to `t_foo_copy_2`.
    ///
    /// If the candidate ID already exists, the number keeps incrementing
    /// until a unique ID is found.
    ///
    /// For `thing_hierarchy`, the duplicate is inserted as a sibling
    /// immediately after the original but with an empty child hierarchy
    /// (children can only have one parent).
    pub fn thing_duplicate(input_diagram: &mut InputDiagram<'static>, thing_id_str: &str) {
        let thing_id_orig = match parse_thing_id(thing_id_str) {
            Some(id) => id,
            None => return,
        };

        let parts = Self::thing_duplicate_id_parts(thing_id_str);

        let (thing_id_dup, dup_number) = {
            let mut n = parts.start_number;
            loop {
                let candidate = parts.format(n);
                if let Some(id) = parse_thing_id(&candidate)
                    && !input_diagram.things.contains_key(&id)
                {
                    break (id, n);
                }
                n += 1;
            }
        };

        // things: insert duplicate in numerically sorted position among
        // siblings that share the same prefix.
        Self::thing_duplicate_in_things(
            input_diagram,
            &thing_id_orig,
            &thing_id_dup,
            &parts,
            dup_number,
        );

        // thing_copy_text: copy entry if it exists.
        Self::thing_duplicate_in_copy_text(
            input_diagram,
            &thing_id_orig,
            &thing_id_dup,
            &parts,
            dup_number,
        );

        // thing_hierarchy: insert as sibling right after the original, with
        // an empty child hierarchy (children can only have one parent).
        Self::thing_duplicate_in_hierarchy(
            &mut input_diagram.thing_hierarchy,
            &thing_id_orig,
            &thing_id_dup,
            &parts,
            dup_number,
        );

        // thing_dependencies: insert duplicate ThingId in numerically
        // sorted position within each EdgeGroup.
        input_diagram
            .thing_dependencies
            .values_mut()
            .for_each(|edge_group| {
                Self::thing_duplicate_in_edge_group(
                    edge_group,
                    &thing_id_orig,
                    &thing_id_dup,
                    &parts,
                    dup_number,
                );
            });

        // thing_interactions: same structure as thing_dependencies.
        input_diagram
            .thing_interactions
            .values_mut()
            .for_each(|edge_group| {
                Self::thing_duplicate_in_edge_group(
                    edge_group,
                    &thing_id_orig,
                    &thing_id_dup,
                    &parts,
                    dup_number,
                );
            });

        // tag_things: add duplicate in numerically sorted position within
        // each tag set that contains the original.
        input_diagram.tag_things.values_mut().for_each(|thing_ids| {
            if thing_ids.contains(&thing_id_orig) {
                Self::thing_duplicate_in_tag_things(
                    thing_ids,
                    &thing_id_orig,
                    &thing_id_dup,
                    &parts,
                    dup_number,
                );
            }
        });

        // entity_descs, entity_tooltips, entity_types, and theme style maps.
        let id_orig = thing_id_orig.into_inner();
        let id_dup = thing_id_dup.into_inner();
        Self::id_copy_insert_in_input_diagram(input_diagram, &id_orig, &id_dup, &parts, dup_number);
    }

    /// Inserts a duplicate entry in the `things` map at the numerically
    /// sorted position among siblings that share the same prefix.
    fn thing_duplicate_in_things(
        diagram: &mut InputDiagram<'static>,
        thing_id_orig: &ThingId<'static>,
        thing_id_dup: &ThingId<'static>,
        parts: &IdDuplicateParts,
        dup_number: u32,
    ) {
        if let Some(orig_index) = diagram.things.get_index_of(thing_id_orig) {
            let name = diagram
                .things
                .get(thing_id_orig)
                .cloned()
                .unwrap_or_default();
            let insert_at = parts.sorted_insert_index(
                diagram
                    .things
                    .keys()
                    .enumerate()
                    .map(|(i, k)| (i, k.as_str())),
                orig_index,
                dup_number,
            );
            diagram
                .things
                .shift_insert(insert_at, thing_id_dup.clone(), name);
        }
    }

    /// Copies a `thing_copy_text` entry (if present) at the numerically
    /// sorted position among siblings that share the same prefix.
    fn thing_duplicate_in_copy_text(
        diagram: &mut InputDiagram<'static>,
        thing_id_orig: &ThingId<'static>,
        thing_id_dup: &ThingId<'static>,
        parts: &IdDuplicateParts,
        dup_number: u32,
    ) {
        if let Some(orig_index) = diagram.thing_copy_text.get_index_of(thing_id_orig) {
            let text = diagram
                .thing_copy_text
                .get(thing_id_orig)
                .cloned()
                .unwrap_or_default();
            let insert_at = parts.sorted_insert_index(
                diagram
                    .thing_copy_text
                    .keys()
                    .enumerate()
                    .map(|(i, k)| (i, k.as_str())),
                orig_index,
                dup_number,
            );
            diagram
                .thing_copy_text
                .shift_insert(insert_at, thing_id_dup.clone(), text);
        }
    }

    /// Inserts the duplicate as a sibling in [`ThingHierarchy`] at the
    /// numerically sorted position, with an empty child hierarchy.
    ///
    /// Searches recursively so the original can be at any nesting depth.
    fn thing_duplicate_in_hierarchy(
        hierarchy: &mut ThingHierarchy<'static>,
        thing_id_orig: &ThingId<'static>,
        thing_id_dup: &ThingId<'static>,
        parts: &IdDuplicateParts,
        dup_number: u32,
    ) -> bool {
        if let Some(orig_index) = hierarchy.get_index_of(thing_id_orig) {
            let insert_at = parts.sorted_insert_index(
                hierarchy.keys().enumerate().map(|(i, k)| (i, k.as_str())),
                orig_index,
                dup_number,
            );
            hierarchy.shift_insert(insert_at, thing_id_dup.clone(), ThingHierarchy::new());
            return true;
        }
        hierarchy.values_mut().any(|child| {
            Self::thing_duplicate_in_hierarchy(
                child,
                thing_id_orig,
                thing_id_dup,
                parts,
                dup_number,
            )
        })
    }

    /// Splits a thing ID string into [`IdDuplicateParts`] for duplicate ID
    /// generation.
    ///
    /// * If the ID ends with `_<digits>` (e.g. `t_something_03`), the prefix
    ///   includes the trailing underscore, the number is incremented by one,
    ///   and `min_width` is the number of digit characters (for leading-zero
    ///   preservation).
    /// * Otherwise `_copy_` is appended and numbering starts at `1`.
    fn thing_duplicate_id_parts(thing_id_str: &str) -> IdDuplicateParts {
        if let Some((before, after)) = thing_id_str.rsplit_once('_')
            && !after.is_empty()
            && after.chars().all(|c| c.is_ascii_digit())
            && let Ok(n) = after.parse::<u32>()
        {
            IdDuplicateParts {
                prefix: format!("{before}_"),
                start_number: n + 1,
                min_width: after.len(),
            }
        } else {
            IdDuplicateParts {
                prefix: format!("{thing_id_str}_copy_"),
                start_number: 1,
                min_width: 1,
            }
        }
    }

    /// Inserts the duplicate `ThingId` in numerically sorted position
    /// inside an [`EdgeGroup`]'s `things` list.
    fn thing_duplicate_in_edge_group(
        edge_group: &mut EdgeGroup<'static>,
        thing_id_orig: &ThingId<'static>,
        thing_id_dup: &ThingId<'static>,
        parts: &IdDuplicateParts,
        dup_number: u32,
    ) {
        if !edge_group.things.contains(thing_id_orig) {
            return;
        }
        let insert_at = parts.sorted_insert_index(
            edge_group
                .things
                .iter()
                .enumerate()
                .map(|(i, k)| (i, k.as_str())),
            edge_group
                .things
                .iter()
                .position(|id| id == thing_id_orig)
                .unwrap_or(0),
            dup_number,
        );
        edge_group.things.insert(insert_at, thing_id_dup.clone());
    }

    /// Inserts the duplicate `ThingId` in numerically sorted position
    /// inside a tag's `Set<ThingId>`.
    fn thing_duplicate_in_tag_things(
        thing_ids: &mut Set<ThingId<'static>>,
        thing_id_orig: &ThingId<'static>,
        thing_id_dup: &ThingId<'static>,
        parts: &IdDuplicateParts,
        dup_number: u32,
    ) {
        let orig_index = match thing_ids.get_index_of(thing_id_orig) {
            Some(idx) => idx,
            None => return,
        };
        let insert_at = parts.sorted_insert_index(
            thing_ids.iter().enumerate().map(|(i, k)| (i, k.as_str())),
            orig_index,
            dup_number,
        );
        thing_ids.shift_insert(insert_at, thing_id_dup.clone());
    }

    /// Copies an [`Id`]'s entries in entity and theme maps, inserting the
    /// duplicate at the numerically sorted position in each map.
    fn id_copy_insert_in_input_diagram(
        input_diagram: &mut InputDiagram<'static>,
        id_orig: &Id<'static>,
        id_dup: &Id<'static>,
        parts: &IdDuplicateParts,
        dup_number: u32,
    ) {
        // entity_descs
        if let Some(orig_index) = input_diagram.entity_descs.get_index_of(id_orig) {
            let value = input_diagram
                .entity_descs
                .get(id_orig)
                .cloned()
                .unwrap_or_default();
            let insert_at = parts.sorted_insert_index(
                input_diagram
                    .entity_descs
                    .keys()
                    .enumerate()
                    .map(|(i, k)| (i, k.as_str())),
                orig_index,
                dup_number,
            );
            input_diagram
                .entity_descs
                .shift_insert(insert_at, id_dup.clone(), value);
        }

        // entity_tooltips
        if let Some(orig_index) = input_diagram.entity_tooltips.get_index_of(id_orig) {
            let value = input_diagram
                .entity_tooltips
                .get(id_orig)
                .cloned()
                .unwrap_or_default();
            let insert_at = parts.sorted_insert_index(
                input_diagram
                    .entity_tooltips
                    .keys()
                    .enumerate()
                    .map(|(i, k)| (i, k.as_str())),
                orig_index,
                dup_number,
            );
            input_diagram
                .entity_tooltips
                .shift_insert(insert_at, id_dup.clone(), value);
        }

        // entity_types
        if let Some(orig_index) = input_diagram.entity_types.get_index_of(id_orig) {
            let value = input_diagram
                .entity_types
                .get(id_orig)
                .cloned()
                .unwrap_or_default();
            let insert_at = parts.sorted_insert_index(
                input_diagram
                    .entity_types
                    .keys()
                    .enumerate()
                    .map(|(i, k)| (i, k.as_str())),
                orig_index,
                dup_number,
            );
            input_diagram
                .entity_types
                .shift_insert(insert_at, id_dup.clone(), value);
        }

        let key_orig = IdOrDefaults::Id(id_orig.clone());
        let key_dup = IdOrDefaults::Id(id_dup.clone());

        // theme_default: base_styles and process_step_selected_styles.
        Self::copy_insert_in_theme_styles(
            &mut input_diagram.theme_default.base_styles,
            &key_orig,
            &key_dup,
            parts,
            dup_number,
        );
        Self::copy_insert_in_theme_styles(
            &mut input_diagram.theme_default.process_step_selected_styles,
            &key_orig,
            &key_dup,
            parts,
            dup_number,
        );

        // theme_types_styles: copy in each ThemeStyles value.
        input_diagram
            .theme_types_styles
            .values_mut()
            .for_each(|theme_styles| {
                Self::copy_insert_in_theme_styles(
                    theme_styles,
                    &key_orig,
                    &key_dup,
                    parts,
                    dup_number,
                );
            });

        // theme_thing_dependencies_styles: both ThemeStyles fields.
        Self::copy_insert_in_theme_styles(
            &mut input_diagram
                .theme_thing_dependencies_styles
                .things_included_styles,
            &key_orig,
            &key_dup,
            parts,
            dup_number,
        );
        Self::copy_insert_in_theme_styles(
            &mut input_diagram
                .theme_thing_dependencies_styles
                .things_excluded_styles,
            &key_orig,
            &key_dup,
            parts,
            dup_number,
        );

        // theme_tag_things_focus: copy in each ThemeStyles value.
        input_diagram
            .theme_tag_things_focus
            .values_mut()
            .for_each(|theme_styles| {
                Self::copy_insert_in_theme_styles(
                    theme_styles,
                    &key_orig,
                    &key_dup,
                    parts,
                    dup_number,
                );
            });
    }

    /// Copies an entry in a [`ThemeStyles`] map, inserting the duplicate
    /// at the numerically sorted position.
    fn copy_insert_in_theme_styles(
        theme_styles: &mut ThemeStyles<'static>,
        key_orig: &IdOrDefaults<'static>,
        key_dup: &IdOrDefaults<'static>,
        parts: &IdDuplicateParts,
        dup_number: u32,
    ) {
        if let Some(orig_index) = theme_styles.get_index_of(key_orig) {
            let value = theme_styles.get(key_orig).cloned().unwrap_or_default();
            let insert_at = parts.sorted_insert_index(
                theme_styles.keys().enumerate().map(|(i, k)| {
                    let s = match k {
                        IdOrDefaults::Id(id) => id.as_str(),
                        _ => "",
                    };
                    (i, s)
                }),
                orig_index,
                dup_number,
            );
            theme_styles.shift_insert(insert_at, key_dup.clone(), value);
        }
    }

    /// Updates the display name for an existing thing.
    pub fn thing_name_update(
        input_diagram: &mut InputDiagram<'static>,
        thing_id_str: &str,
        name: &str,
    ) {
        if let Some(thing_id) = parse_thing_id(thing_id_str)
            && let Some(entry) = input_diagram.things.get_mut(&thing_id)
        {
            *entry = name.to_owned();
        }
    }

    /// Renames a thing across all maps in the [`InputDiagram`].
    pub fn thing_rename(
        input_diagram: &mut InputDiagram<'static>,
        thing_id_old_str: &str,
        thing_id_new_str: &str,
    ) {
        if thing_id_old_str == thing_id_new_str {
            return;
        }

        if let Ok(thing_id_old) = Id::new(thing_id_old_str)
            .map(Id::into_static)
            .map(ThingId::from)
            && let Ok(thing_id_new) = Id::new(thing_id_new_str)
                .map(Id::into_static)
                .map(ThingId::from)
        {
            // things: rename ThingId key.
            if let Some(thing_index) = input_diagram.things.get_index_of(&thing_id_old) {
                let _result = input_diagram
                    .things
                    .replace_index(thing_index, thing_id_new.clone());
            }

            // thing_copy_text: rename ThingId key.
            if let Some(thing_index) = input_diagram.thing_copy_text.get_index_of(&thing_id_old) {
                let _result = input_diagram
                    .thing_copy_text
                    .replace_index(thing_index, thing_id_new.clone());
            }

            // thing_hierarchy: recursive rename.
            if let Some((thing_hierarchy_with_id, thing_index)) =
                Self::thing_rename_in_hierarchy(&mut input_diagram.thing_hierarchy, &thing_id_old)
            {
                let _result =
                    thing_hierarchy_with_id.replace_index(thing_index, thing_id_new.clone());
            }

            // thing_dependencies: rename ThingIds inside EdgeGroup values.
            input_diagram
                .thing_dependencies
                .values_mut()
                .for_each(|edge_group| {
                    Self::thing_rename_in_edge_group(edge_group, &thing_id_old, &thing_id_new);
                });

            // thing_interactions: same structure as thing_dependencies.
            input_diagram
                .thing_interactions
                .values_mut()
                .for_each(|edge_group| {
                    Self::thing_rename_in_edge_group(edge_group, &thing_id_old, &thing_id_new);
                });

            // tag_things: rename ThingIds in each Set<ThingId> value.
            input_diagram.tag_things.values_mut().for_each(
                |thing_ids: &mut Set<ThingId<'static>>| {
                    if let Some(index) = thing_ids.get_index_of(&thing_id_old) {
                        let _result = thing_ids.replace_index(index, thing_id_new.clone());
                    }
                },
            );

            // Shared rename across entity_descs, entity_tooltips, entity_types,
            // and all theme style maps.
            let id_old = thing_id_old.into_inner();
            let id_new = thing_id_new.into_inner();
            id_rename_in_input_diagram(input_diagram, &id_old, &id_new);
        }
    }

    /// Replaces occurrences of `thing_id_old` with `thing_id_new` inside an
    /// [`EdgeGroup`] (which contains a `Vec<ThingId>`).
    fn thing_rename_in_edge_group(
        edge_group: &mut EdgeGroup<'static>,
        thing_id_old: &ThingId<'static>,
        thing_id_new: &ThingId<'static>,
    ) {
        edge_group.things.iter_mut().for_each(|thing_id| {
            if thing_id == thing_id_old {
                *thing_id = thing_id_new.clone();
            }
        });
    }

    /// Searches recursively through a [`ThingHierarchy`] for a given
    /// [`ThingId`] key, returning a mutable reference to the containing map
    /// and the index within it.
    fn thing_rename_in_hierarchy<'f, 'id>(
        thing_hierarchy: &'f mut ThingHierarchy<'id>,
        thing_id: &'f ThingId<'id>,
    ) -> Option<(&'f mut ThingHierarchy<'id>, usize)> {
        if let Some(thing_index) = thing_hierarchy.get_index_of(thing_id) {
            Some((thing_hierarchy, thing_index))
        } else {
            thing_hierarchy
                .values_mut()
                .find_map(|thing_hierarchy_child| {
                    Self::thing_rename_in_hierarchy(thing_hierarchy_child, thing_id)
                })
        }
    }

    /// Removes a thing and all references to it from the [`InputDiagram`].
    ///
    /// Uses `shift_remove` to preserve ordering of remaining entries.
    pub fn thing_remove(input_diagram: &mut InputDiagram<'static>, thing_id_str: &str) {
        if let Some(thing_id) = parse_thing_id(thing_id_str) {
            // things: remove ThingId key.
            input_diagram.things.shift_remove(&thing_id);

            // thing_copy_text: remove ThingId key.
            input_diagram.thing_copy_text.shift_remove(&thing_id);

            // thing_hierarchy: recursive remove.
            Self::thing_remove_from_hierarchy(&mut input_diagram.thing_hierarchy, &thing_id);

            // thing_dependencies: remove ThingId from EdgeGroup values.
            input_diagram
                .thing_dependencies
                .values_mut()
                .for_each(|edge_group| {
                    Self::thing_remove_from_edge_group(edge_group, &thing_id);
                });

            // thing_interactions: same structure as thing_dependencies.
            input_diagram
                .thing_interactions
                .values_mut()
                .for_each(|edge_group| {
                    Self::thing_remove_from_edge_group(edge_group, &thing_id);
                });

            // tag_things: remove ThingId from each Set<ThingId> value.
            input_diagram.tag_things.values_mut().for_each(
                |thing_ids: &mut Set<ThingId<'static>>| {
                    thing_ids.shift_remove(&thing_id);
                },
            );

            // entity_descs, entity_tooltips, entity_types, and theme style
            // maps: remove by Id.
            let id = thing_id.into_inner();
            Self::thing_remove_id_from_input_diagram(input_diagram, &id);
        }
    }

    /// Removes an [`Id`] from entity and theme maps in the
    /// [`InputDiagram`].
    fn thing_remove_id_from_input_diagram(
        input_diagram: &mut InputDiagram<'static>,
        id: &Id<'static>,
    ) {
        input_diagram.entity_descs.shift_remove(id);
        input_diagram.entity_tooltips.shift_remove(id);
        input_diagram.entity_types.shift_remove(id);

        let key = IdOrDefaults::Id(id.clone());

        // theme_default: remove from base_styles and
        // process_step_selected_styles.
        input_diagram.theme_default.base_styles.shift_remove(&key);
        input_diagram
            .theme_default
            .process_step_selected_styles
            .shift_remove(&key);

        // theme_types_styles: remove from each ThemeStyles value.
        input_diagram
            .theme_types_styles
            .values_mut()
            .for_each(|theme_styles| {
                theme_styles.shift_remove(&key);
            });

        // theme_thing_dependencies_styles: remove from both ThemeStyles
        // fields.
        input_diagram
            .theme_thing_dependencies_styles
            .things_included_styles
            .shift_remove(&key);
        input_diagram
            .theme_thing_dependencies_styles
            .things_excluded_styles
            .shift_remove(&key);

        // theme_tag_things_focus: remove from each ThemeStyles value.
        input_diagram
            .theme_tag_things_focus
            .values_mut()
            .for_each(|theme_styles| {
                theme_styles.shift_remove(&key);
            });
    }

    /// Removes all occurrences of `thing_id` from an [`EdgeGroup`]'s
    /// `things` list.
    fn thing_remove_from_edge_group(
        edge_group: &mut EdgeGroup<'static>,
        thing_id: &ThingId<'static>,
    ) {
        edge_group.things.retain(|id| id != thing_id);
    }

    /// Removes a `ThingId` key from a [`ThingHierarchy`], re-parenting its
    /// children into the same level at the position it occupied.
    ///
    /// Searches recursively through all nesting levels. Returns `true` if
    /// the key was found and removed.
    fn thing_remove_from_hierarchy(
        hierarchy: &mut ThingHierarchy<'static>,
        thing_id: &ThingId<'static>,
    ) -> bool {
        if let Some(removal_index) = hierarchy.get_index_of(thing_id) {
            let children = hierarchy.shift_remove(thing_id).unwrap_or_default();

            // Insert children at the position where the removed node was,
            // preserving their original order.
            children.into_inner().into_iter().enumerate().for_each(
                |(offset, (child_id, child_hierarchy))| {
                    hierarchy.shift_insert(removal_index + offset, child_id, child_hierarchy);
                },
            );

            return true;
        }
        hierarchy
            .values_mut()
            .any(|child| Self::thing_remove_from_hierarchy(child, thing_id))
    }

    /// Moves a thing entry from one index to another in the `things` map.
    pub fn thing_move(input_diagram: &mut InputDiagram<'static>, from: usize, to: usize) {
        input_diagram.things.move_index(from, to);
    }

    // === Copy text helpers === //

    /// Adds a new copy-text row with a unique placeholder ThingId.
    pub fn copy_text_add(input_diagram: &mut InputDiagram<'static>) {
        let mut n = input_diagram.thing_copy_text.len();
        loop {
            let candidate = format!("thing_{n}");
            if let Some(thing_id) = parse_thing_id(&candidate)
                && !input_diagram.thing_copy_text.contains_key(&thing_id)
            {
                input_diagram
                    .thing_copy_text
                    .insert(thing_id, String::new());
                break;
            }
            n += 1;
        }
    }

    /// Adds a new entity description row with a unique placeholder Id.
    pub fn entity_desc_add(input_diagram: &mut InputDiagram<'static>) {
        let mut n = input_diagram.entity_descs.len();
        loop {
            let candidate = format!("entity_{n}");
            if let Some(id) = parse_id(&candidate)
                && !input_diagram.entity_descs.contains_key(&id)
            {
                input_diagram.entity_descs.insert(id, String::new());
                break;
            }
            n += 1;
        }
    }

    /// Adds a new entity tooltip row with a unique placeholder Id.
    pub fn entity_tooltip_add(input_diagram: &mut InputDiagram<'static>) {
        let mut n = input_diagram.entity_tooltips.len();
        loop {
            let candidate = format!("entity_{n}");
            if let Some(id) = parse_id(&candidate)
                && !input_diagram.entity_tooltips.contains_key(&id)
            {
                input_diagram.entity_tooltips.insert(id, String::new());
                break;
            }
            n += 1;
        }
    }

    // === Key-value (copy-text / desc / tooltip) mutation helpers === //

    /// Renames the key of a key-value entry in the target map.
    pub fn kv_entry_rename(
        input_diagram: &mut InputDiagram<'static>,
        target: OnChangeTarget,
        id_old_str: &str,
        id_new_str: &str,
        current_value: &str,
    ) {
        if id_old_str == id_new_str {
            return;
        }
        match target {
            OnChangeTarget::CopyText => {
                let thing_id_old = match parse_thing_id(id_old_str) {
                    Some(id) => id,
                    None => return,
                };
                let thing_id_new = match parse_thing_id(id_new_str) {
                    Some(id) => id,
                    None => return,
                };
                input_diagram
                    .thing_copy_text
                    .insert(thing_id_new, current_value.to_owned());
                input_diagram.thing_copy_text.swap_remove(&thing_id_old);
            }
            OnChangeTarget::EntityDesc => {
                let id_old = match parse_id(id_old_str) {
                    Some(id) => id,
                    None => return,
                };
                let id_new = match parse_id(id_new_str) {
                    Some(id) => id,
                    None => return,
                };
                input_diagram
                    .entity_descs
                    .insert(id_new, current_value.to_owned());
                input_diagram.entity_descs.swap_remove(&id_old);
            }
            OnChangeTarget::EntityTooltip => {
                let id_old = match parse_id(id_old_str) {
                    Some(id) => id,
                    None => return,
                };
                let id_new = match parse_id(id_new_str) {
                    Some(id) => id,
                    None => return,
                };
                input_diagram
                    .entity_tooltips
                    .insert(id_new, current_value.to_owned());
                input_diagram.entity_tooltips.swap_remove(&id_old);
            }
        }
    }

    /// Updates the value of a key-value entry in the target map.
    pub fn kv_entry_update(
        input_diagram: &mut InputDiagram<'static>,
        target: OnChangeTarget,
        id_str: &str,
        value: &str,
    ) {
        match target {
            OnChangeTarget::CopyText => {
                if let Some(thing_id) = parse_thing_id(id_str)
                    && let Some(entry) = input_diagram.thing_copy_text.get_mut(&thing_id)
                {
                    *entry = value.to_owned();
                }
            }
            OnChangeTarget::EntityDesc => {
                if let Some(entity_id) = parse_id(id_str)
                    && let Some(entry) = input_diagram.entity_descs.get_mut(&entity_id)
                {
                    *entry = value.to_owned();
                }
            }
            OnChangeTarget::EntityTooltip => {
                if let Some(entity_id) = parse_id(id_str)
                    && let Some(entry) = input_diagram.entity_tooltips.get_mut(&entity_id)
                {
                    *entry = value.to_owned();
                }
            }
        }
    }

    /// Removes a key-value entry from the target map.
    pub fn kv_entry_remove(
        input_diagram: &mut InputDiagram<'static>,
        target: OnChangeTarget,
        id_str: &str,
    ) {
        match target {
            OnChangeTarget::CopyText => {
                if let Some(thing_id) = parse_thing_id(id_str) {
                    input_diagram.thing_copy_text.shift_remove(&thing_id);
                }
            }
            OnChangeTarget::EntityDesc => {
                if let Some(entity_id) = parse_id(id_str) {
                    input_diagram.entity_descs.shift_remove(&entity_id);
                }
            }
            OnChangeTarget::EntityTooltip => {
                if let Some(entity_id) = parse_id(id_str) {
                    input_diagram.entity_tooltips.shift_remove(&entity_id);
                }
            }
        }
    }

    /// Moves a key-value entry from one index to another in the target map.
    pub fn kv_entry_move(
        input_diagram: &mut InputDiagram<'static>,
        target: OnChangeTarget,
        from: usize,
        to: usize,
    ) {
        match target {
            OnChangeTarget::CopyText => input_diagram.thing_copy_text.move_index(from, to),
            OnChangeTarget::EntityDesc => input_diagram.entity_descs.move_index(from, to),
            OnChangeTarget::EntityTooltip => input_diagram.entity_tooltips.move_index(from, to),
        }
    }
}
