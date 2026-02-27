//! Card-based editor for [`ThemeStyles`] maps.
//!
//! Instead of a raw YAML textarea, this module provides:
//!
//! - [`ThemeStylesEditor`]: iterates over a `ThemeStyles` map and renders one
//!   [`CssClassPartialsCard`] per entry, plus an "add" button.
//! - [`CssClassPartialsCard`]: renders the key (`IdOrDefaults`) as a `<select>`
//!   / text input, the `style_aliases_applied` list, and each `ThemeAttr →
//!   value` pair with individual inputs.
//!
//! The [`ThemeStylesTarget`] enum tells the editor which field inside
//! [`InputDiagram`] to read from / write to. Variants exist for:
//!
//! - `theme_default.base_styles`
//! - `theme_default.process_step_selected_styles`
//! - `theme_types_styles[entity_type_key]`
//! - `theme_thing_dependencies_styles.things_included_styles`
//! - `theme_thing_dependencies_styles.things_excluded_styles`
//! - `theme_tag_things_focus[tag_key]`

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::{
    input_model::{
        theme::{CssClassPartials, IdOrDefaults, StyleAlias, ThemeAttr, ThemeStyles},
        InputDiagram,
    },
    model_common::Id,
};

use crate::components::editor::{
    common::{
        ADD_BTN, CARD_CLASS, INPUT_CLASS, LABEL_CLASS, REMOVE_BTN, ROW_CLASS_SIMPLE, SELECT_CLASS,
    },
    datalists::list_ids,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// All `ThemeAttr` variants paired with their `snake_case` serialisation name.
///
/// `ThemeAttr` derives `Serialize` with `#[serde(rename_all = "snake_case")]`
/// but exposes no iteration helper, so we maintain a static list.
const THEME_ATTRS: &[(&str, ThemeAttr)] = &[
    ("animate", ThemeAttr::Animate),
    ("cursor", ThemeAttr::Cursor),
    ("circle_radius", ThemeAttr::CircleRadius),
    ("extra", ThemeAttr::Extra),
    ("fill_color", ThemeAttr::FillColor),
    ("fill_color_normal", ThemeAttr::FillColorNormal),
    ("fill_color_focus", ThemeAttr::FillColorFocus),
    ("fill_color_hover", ThemeAttr::FillColorHover),
    ("fill_color_active", ThemeAttr::FillColorActive),
    ("fill_shade", ThemeAttr::FillShade),
    ("fill_shade_normal", ThemeAttr::FillShadeNormal),
    ("fill_shade_focus", ThemeAttr::FillShadeFocus),
    ("fill_shade_hover", ThemeAttr::FillShadeHover),
    ("fill_shade_active", ThemeAttr::FillShadeActive),
    ("gap", ThemeAttr::Gap),
    ("padding", ThemeAttr::Padding),
    ("padding_x", ThemeAttr::PaddingX),
    ("padding_y", ThemeAttr::PaddingY),
    ("padding_left", ThemeAttr::PaddingLeft),
    ("padding_right", ThemeAttr::PaddingRight),
    ("padding_top", ThemeAttr::PaddingTop),
    ("padding_bottom", ThemeAttr::PaddingBottom),
    ("margin", ThemeAttr::Margin),
    ("margin_x", ThemeAttr::MarginX),
    ("margin_y", ThemeAttr::MarginY),
    ("margin_left", ThemeAttr::MarginLeft),
    ("margin_right", ThemeAttr::MarginRight),
    ("margin_top", ThemeAttr::MarginTop),
    ("margin_bottom", ThemeAttr::MarginBottom),
    ("opacity", ThemeAttr::Opacity),
    ("outline_color", ThemeAttr::OutlineColor),
    ("outline_color_normal", ThemeAttr::OutlineColorNormal),
    ("outline_color_focus", ThemeAttr::OutlineColorFocus),
    ("outline_color_hover", ThemeAttr::OutlineColorHover),
    ("outline_color_active", ThemeAttr::OutlineColorActive),
    ("outline_shade", ThemeAttr::OutlineShade),
    ("outline_shade_normal", ThemeAttr::OutlineShadeNormal),
    ("outline_shade_focus", ThemeAttr::OutlineShadeFocus),
    ("outline_shade_hover", ThemeAttr::OutlineShadeHover),
    ("outline_shade_active", ThemeAttr::OutlineShadeActive),
    ("outline_width", ThemeAttr::OutlineWidth),
    ("outline_style", ThemeAttr::OutlineStyle),
    ("outline_style_normal", ThemeAttr::OutlineStyleNormal),
    ("outline_style_focus", ThemeAttr::OutlineStyleFocus),
    ("outline_style_hover", ThemeAttr::OutlineStyleHover),
    ("outline_style_active", ThemeAttr::OutlineStyleActive),
    ("radius_top_left", ThemeAttr::RadiusTopLeft),
    ("radius_top_right", ThemeAttr::RadiusTopRight),
    ("radius_bottom_left", ThemeAttr::RadiusBottomLeft),
    ("radius_bottom_right", ThemeAttr::RadiusBottomRight),
    ("shape_color", ThemeAttr::ShapeColor),
    ("stroke_color", ThemeAttr::StrokeColor),
    ("stroke_color_normal", ThemeAttr::StrokeColorNormal),
    ("stroke_color_focus", ThemeAttr::StrokeColorFocus),
    ("stroke_color_hover", ThemeAttr::StrokeColorHover),
    ("stroke_color_active", ThemeAttr::StrokeColorActive),
    ("stroke_shade", ThemeAttr::StrokeShade),
    ("stroke_shade_normal", ThemeAttr::StrokeShadeNormal),
    ("stroke_shade_focus", ThemeAttr::StrokeShadeFocus),
    ("stroke_shade_hover", ThemeAttr::StrokeShadeHover),
    ("stroke_shade_active", ThemeAttr::StrokeShadeActive),
    ("stroke_width", ThemeAttr::StrokeWidth),
    ("stroke_style", ThemeAttr::StrokeStyle),
    ("stroke_style_normal", ThemeAttr::StrokeStyleNormal),
    ("stroke_style_focus", ThemeAttr::StrokeStyleFocus),
    ("stroke_style_hover", ThemeAttr::StrokeStyleHover),
    ("stroke_style_active", ThemeAttr::StrokeStyleActive),
    ("text_color", ThemeAttr::TextColor),
    ("text_shade", ThemeAttr::TextShade),
    ("visibility", ThemeAttr::Visibility),
];

/// Well-known keys for the `IdOrDefaults` select dropdown.
const ID_OR_DEFAULTS_BUILTINS: &[(&str, &str)] = &[
    ("node_defaults", "Node Defaults"),
    ("node_excluded_defaults", "Node Excluded Defaults"),
    ("edge_defaults", "Edge Defaults"),
];

// ---------------------------------------------------------------------------
// Snapshot type alias (avoids clippy::type_complexity)
// ---------------------------------------------------------------------------

/// `(key, style_aliases_applied, Vec<(attr_name, attr_value)>)` snapshot.
type EntrySnapshot = (String, Vec<String>, Vec<(String, String)>);

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Look up the `snake_case` name for a `ThemeAttr`.
fn theme_attr_name(attr: &ThemeAttr) -> &'static str {
    THEME_ATTRS
        .iter()
        .find(|(_, a)| a == attr)
        .map(|(name, _)| *name)
        .unwrap_or("unknown")
}

/// Parse a string into an `IdOrDefaults`.
fn parse_id_or_defaults(s: &str) -> Option<IdOrDefaults<'static>> {
    match s {
        "node_defaults" => Some(IdOrDefaults::NodeDefaults),
        "node_excluded_defaults" => Some(IdOrDefaults::NodeExcludedDefaults),
        "edge_defaults" => Some(IdOrDefaults::EdgeDefaults),
        other => Id::new(other)
            .ok()
            .map(|id| IdOrDefaults::Id(id.into_static())),
    }
}

/// Parse a string into a `ThemeAttr` using the static table.
fn parse_theme_attr(s: &str) -> Option<ThemeAttr> {
    THEME_ATTRS
        .iter()
        .find(|(name, _)| *name == s)
        .map(|(_, attr)| *attr)
}

// ===========================================================================
// Public: ThemeStylesTarget
// ===========================================================================

/// Which field of [`InputDiagram`] this editor targets.
///
/// The editor needs to know where to read/write the `ThemeStyles` map inside
/// the diagram. Each variant corresponds to one field path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThemeStylesTarget {
    /// `theme_default.base_styles`
    BaseStyles,
    /// `theme_default.process_step_selected_styles`
    ProcessStepSelectedStyles,
    /// `theme_types_styles[entity_type_key]` — styles for a particular entity
    /// type.
    TypesStyles {
        /// The `EntityTypeId` key as a string (e.g. `"type_organisation"`).
        entity_type_key: String,
    },
    /// `theme_thing_dependencies_styles.things_included_styles`
    DependenciesIncluded,
    /// `theme_thing_dependencies_styles.things_excluded_styles`
    DependenciesExcluded,
    /// `theme_tag_things_focus[tag_key]` — styles for a particular tag (or
    /// `tag_defaults`).
    TagFocus {
        /// The `TagIdOrDefaults` key as a string (e.g. `"tag_defaults"` or
        /// `"tag_app_development"`).
        tag_key: String,
    },
}

impl ThemeStylesTarget {
    /// Read the [`ThemeStyles`] from the diagram.
    ///
    /// Returns `None` when the outer map key does not (yet) exist for
    /// [`TypesStyles`] or [`TagFocus`] variants.
    fn read<'diag>(
        &self,
        diagram: &'diag InputDiagram<'static>,
    ) -> Option<&'diag ThemeStyles<'static>> {
        match self {
            Self::BaseStyles => Some(&diagram.theme_default.base_styles),
            Self::ProcessStepSelectedStyles => {
                Some(&diagram.theme_default.process_step_selected_styles)
            }
            Self::TypesStyles { entity_type_key } => {
                let type_id = parse_entity_type_id(entity_type_key)?;
                diagram.theme_types_styles.get(&type_id)
            }
            Self::DependenciesIncluded => Some(
                &diagram
                    .theme_thing_dependencies_styles
                    .things_included_styles,
            ),
            Self::DependenciesExcluded => Some(
                &diagram
                    .theme_thing_dependencies_styles
                    .things_excluded_styles,
            ),
            Self::TagFocus { tag_key } => {
                let tag = parse_tag_id_or_defaults(tag_key)?;
                diagram.theme_tag_things_focus.get(&tag)
            }
        }
    }

    /// Obtain a mutable reference to the [`ThemeStyles`] inside the diagram.
    ///
    /// For [`TypesStyles`] and [`TagFocus`], the entry is inserted with a
    /// default value if it does not yet exist.
    fn write_mut<'diag>(
        &self,
        diagram: &'diag mut InputDiagram<'static>,
    ) -> Option<&'diag mut ThemeStyles<'static>> {
        match self {
            Self::BaseStyles => Some(&mut diagram.theme_default.base_styles),
            Self::ProcessStepSelectedStyles => {
                Some(&mut diagram.theme_default.process_step_selected_styles)
            }
            Self::TypesStyles { entity_type_key } => {
                let type_id = parse_entity_type_id(entity_type_key)?;
                // Use entry API to insert default if missing.
                Some(
                    diagram
                        .theme_types_styles
                        .entry(type_id)
                        .or_insert_with(ThemeStyles::default),
                )
            }
            Self::DependenciesIncluded => Some(
                &mut diagram
                    .theme_thing_dependencies_styles
                    .things_included_styles,
            ),
            Self::DependenciesExcluded => Some(
                &mut diagram
                    .theme_thing_dependencies_styles
                    .things_excluded_styles,
            ),
            Self::TagFocus { tag_key } => {
                let tag = parse_tag_id_or_defaults(tag_key)?;
                Some(
                    diagram
                        .theme_tag_things_focus
                        .entry(tag)
                        .or_insert_with(ThemeStyles::default),
                )
            }
        }
    }
}

/// Parse a string into an `EntityTypeId<'static>`.
fn parse_entity_type_id(
    s: &str,
) -> Option<disposition::model_common::entity::EntityTypeId<'static>> {
    use disposition::model_common::entity::EntityTypeId;
    Id::new(s)
        .ok()
        .map(|id| EntityTypeId::from(id.into_static()))
}

/// Parse a string into a `TagIdOrDefaults<'static>`.
fn parse_tag_id_or_defaults(
    s: &str,
) -> Option<disposition::input_model::theme::TagIdOrDefaults<'static>> {
    use disposition::input_model::{tag::TagId, theme::TagIdOrDefaults};
    match s {
        "tag_defaults" => Some(TagIdOrDefaults::TagDefaults),
        other => Id::new(other)
            .ok()
            .map(|id| TagIdOrDefaults::Custom(TagId::from(id.into_static()))),
    }
}

// ===========================================================================
// Public: ThemeStylesEditor
// ===========================================================================

/// Card-based editor for a [`ThemeStyles`] map.
///
/// Renders one [`CssClassPartialsCard`] per `IdOrDefaults → CssClassPartials`
/// entry, plus an "+ Add entry" button at the bottom.
#[component]
pub fn ThemeStylesEditor(
    input_diagram: Signal<InputDiagram<'static>>,
    target: ThemeStylesTarget,
) -> Element {
    let diagram = input_diagram.read();
    let theme_styles = target.read(&diagram);

    // If the target doesn't exist yet (e.g. a types-styles key was just
    // removed), render an empty placeholder.
    let Some(theme_styles) = theme_styles else {
        drop(diagram);
        return rsx! {
            div {
                class: "flex flex-col gap-2",
                div {
                    class: ADD_BTN,
                    onclick: {
                        let target = target.clone();
                        move |_| {
                            let mut diagram = input_diagram.write();
                            if let Some(styles) = target.write_mut(&mut diagram) {
                                styles.insert(
                                    IdOrDefaults::NodeDefaults,
                                    CssClassPartials::default(),
                                );
                            }
                        }
                    },
                    "+ Add entry"
                }
            }
        };
    };

    // Snapshot the entries so we can drop the borrow before event handlers.
    let entries: Vec<EntrySnapshot> = theme_styles
        .iter()
        .map(
            |(key, css_partials): (&IdOrDefaults<'static>, &CssClassPartials<'static>)| {
                let key_str = key.as_str().to_owned();
                let aliases: Vec<String> = css_partials
                    .style_aliases_applied
                    .iter()
                    .map(|a: &StyleAlias<'static>| a.as_str().to_owned())
                    .collect();
                let attrs: Vec<(String, String)> = css_partials
                    .partials
                    .iter()
                    .map(|(attr, val): (&ThemeAttr, &String)| {
                        (theme_attr_name(attr).to_owned(), val.clone())
                    })
                    .collect();
                (key_str, aliases, attrs)
            },
        )
        .collect();
    drop(diagram);

    rsx! {
        div {
            class: "flex flex-col gap-2",

            for (idx, entry) in entries.iter().enumerate() {
                {
                    let key = entry.0.clone();
                    let aliases = entry.1.clone();
                    let attrs = entry.2.clone();
                    let target = target.clone();
                    rsx! {
                        CssClassPartialsCard {
                            key: "entry_{idx}_{key}",
                            input_diagram,
                            target,
                            entry_index: idx,
                            entry_key: key,
                            style_aliases: aliases,
                            theme_attrs: attrs,
                        }
                    }
                }
            }

            div {
                class: ADD_BTN,
                onclick: {
                    let target = target.clone();
                    move |_| {
                        let mut diagram = input_diagram.write();
                        let Some(styles) = target.write_mut(&mut diagram) else {
                            return;
                        };
                        // Find a key that doesn't exist yet.
                        let new_key = if !styles.contains_key(&IdOrDefaults::NodeDefaults) {
                            IdOrDefaults::NodeDefaults
                        } else if !styles.contains_key(&IdOrDefaults::EdgeDefaults) {
                            IdOrDefaults::EdgeDefaults
                        } else {
                            // Generate a placeholder custom ID.
                            let mut n = 1u32;
                            loop {
                                let candidate = format!("custom_{n}");
                                if let Some(id) = parse_id_or_defaults(&candidate)
                                    && !styles.contains_key(&id)
                                {
                                    break id;
                                }
                                n += 1;
                            }
                        };
                        styles.insert(new_key, CssClassPartials::default());
                    }
                },
                "+ Add entry"
            }
        }
    }
}

// ===========================================================================
// CssClassPartialsCard
// ===========================================================================

/// A single card within the [`ThemeStylesEditor`].
///
/// Shows:
/// 1. **Header** — key selector (select for built-ins, text for custom IDs) and
///    a remove button.
/// 2. **Style aliases** — list of applied aliases with remove buttons + add.
/// 3. **Theme attributes** — key/value rows with remove buttons + add.
#[component]
fn CssClassPartialsCard(
    input_diagram: Signal<InputDiagram<'static>>,
    target: ThemeStylesTarget,
    entry_index: usize,
    entry_key: String,
    style_aliases: Vec<String>,
    theme_attrs: Vec<(String, String)>,
) -> Element {
    let is_builtin = matches!(
        entry_key.as_str(),
        "node_defaults" | "node_excluded_defaults" | "edge_defaults"
    );

    rsx! {
        div {
            class: CARD_CLASS,

            // ── Header row: key + remove ─────────────────────────────
            div {
                class: ROW_CLASS_SIMPLE,

                label {
                    class: "text-xs text-gray-500 w-10 shrink-0",
                    "Key"
                }

                if is_builtin {
                    select {
                        class: SELECT_CLASS,
                        value: "{entry_key}",
                        onchange: {
                            let old_key = entry_key.clone();
                            let target = target.clone();
                            move |evt: dioxus::events::FormEvent| {
                                let new_val = evt.value();
                                if let (Some(old), Some(new)) = (
                                    parse_id_or_defaults(&old_key),
                                    parse_id_or_defaults(&new_val),
                                )
                                    && old != new
                                {
                                    let mut diagram = input_diagram.write();
                                    let Some(styles) = target.write_mut(&mut diagram) else {
                                        return;
                                    };
                                    if let Some(idx) = styles.get_index_of(&old) {
                                        styles
                                            .replace_index(idx, new)
                                            .expect("Expected new key to be unique after equality check");
                                    }
                                }
                            }
                        },

                        for (val, label) in ID_OR_DEFAULTS_BUILTINS.iter() {
                            option {
                                value: "{val}",
                                selected: *val == entry_key.as_str(),
                                "{label}"
                            }
                        }
                    }
                } else {
                    input {
                        class: INPUT_CLASS,
                        style: "max-width:14rem",
                        list: list_ids::ENTITY_IDS,
                        placeholder: "entity_id",
                        value: "{entry_key}",
                        onchange: {
                            let old_key = entry_key.clone();
                            let target = target.clone();
                            move |evt: dioxus::events::FormEvent| {
                                let new_val = evt.value();
                                if let (Some(old), Some(new)) = (
                                    parse_id_or_defaults(&old_key),
                                    parse_id_or_defaults(&new_val),
                                )
                                    && old != new
                                {
                                    let mut diagram = input_diagram.write();
                                    let Some(styles) = target.write_mut(&mut diagram) else {
                                        return;
                                    };
                                    if let Some(idx) = styles.get_index_of(&old) {
                                        styles
                                            .replace_index(idx, new)
                                            .expect("Expected new key to be unique after equality check");
                                    }
                                }
                            }
                        },
                    }
                }

                // Toggle to switch between builtin <select> and custom <input>.
                label {
                    class: "text-xs text-gray-500 ml-1 flex items-center gap-1 select-none cursor-pointer",
                    title: "Toggle between built-in defaults and a custom entity ID",
                    input {
                        r#type: "checkbox",
                        class: "accent-blue-500",
                        checked: !is_builtin,
                        onchange: {
                            let old_key = entry_key.clone();
                            let target = target.clone();
                            move |evt: dioxus::events::FormEvent| {
                                let wants_custom = evt.value() == "true";
                                let new_key = if wants_custom {
                                    // Switch from built-in to custom placeholder.
                                    let mut n = 1u32;
                                    loop {
                                        let candidate = format!("custom_{n}");
                                        if let Some(id) = parse_id_or_defaults(&candidate) {
                                            let diagram = input_diagram.read();
                                            let styles = target.read(&diagram);
                                            if let Some(styles) = styles {
                                                if !styles.contains_key(&id) {
                                                    drop(diagram);
                                                    break Some(id);
                                                }
                                            }
                                            drop(diagram);
                                        }
                                        n += 1;
                                    }
                                } else {
                                    // Switch from custom to first available built-in.
                                    let diagram = input_diagram.read();
                                    let styles = target.read(&diagram);
                                    let key = styles.and_then(|styles| {
                                        [
                                            IdOrDefaults::NodeDefaults,
                                            IdOrDefaults::NodeExcludedDefaults,
                                            IdOrDefaults::EdgeDefaults,
                                        ]
                                        .into_iter()
                                        .find(|k| !styles.contains_key(k))
                                    });
                                    drop(diagram);
                                    key
                                };
                                if let Some(new) = new_key
                                    && let Some(old) = parse_id_or_defaults(&old_key)
                                {
                                    let mut diagram = input_diagram.write();
                                    let Some(styles) = target.write_mut(&mut diagram) else {
                                        return;
                                    };
                                    if let Some(idx) = styles.get_index_of(&old) {
                                        styles
                                            .replace_index(idx, new)
                                            .expect("Expected new key to be unique; checked for availability above");
                                    }
                                }
                            }
                        },
                    }
                    "ID"
                }

                span {
                    class: REMOVE_BTN,
                    onclick: {
                        let key = entry_key.clone();
                        let target = target.clone();
                        move |_| {
                            if let Some(parsed) = parse_id_or_defaults(&key) {
                                let mut diagram = input_diagram.write();
                                let Some(styles) = target.write_mut(&mut diagram) else {
                                    return;
                                };
                                styles.shift_remove(&parsed);
                            }
                        }
                    },
                    "✕ Remove"
                }
            }

            // ── Style aliases applied ────────────────────────────────
            div {
                class: "flex flex-col gap-1 pl-4",

                label {
                    class: LABEL_CLASS,
                    "Style aliases applied"
                }

                for (alias_idx, alias_name) in style_aliases.iter().enumerate() {
                    {
                        let alias_name = alias_name.clone();
                        let key = entry_key.clone();
                        let target = target.clone();
                        rsx! {
                            div {
                                key: "alias_{alias_idx}_{alias_name}",
                                class: ROW_CLASS_SIMPLE,

                                input {
                                    class: INPUT_CLASS,
                                    style: "max-width:12rem",
                                    list: list_ids::STYLE_ALIASES,
                                    placeholder: "style_alias",
                                    value: "{alias_name}",
                                    onchange: {
                                        let key = key.clone();
                                        let target = target.clone();
                                        move |evt: dioxus::events::FormEvent| {
                                            let new_val = evt.value();
                                            if let Some(parsed_key) = parse_id_or_defaults(&key) {
                                                // Parse the alias through serde round-trip:
                                                // StyleAlias::from(Id) handles builtin matching.
                                                if let Ok(new_alias_id) = Id::new(&new_val) {
                                                    let new_alias = StyleAlias::from(new_alias_id.into_static()).into_static();
                                                    let mut diagram = input_diagram.write();
                                                    let Some(styles) = target.write_mut(&mut diagram) else {
                                                        return;
                                                    };
                                                    if let Some(partials) = styles.get_mut(&parsed_key)
                                                        && alias_idx < partials.style_aliases_applied.len()
                                                    {
                                                        partials.style_aliases_applied[alias_idx] = new_alias;
                                                    }
                                                }
                                            }
                                        }
                                    },
                                }

                                span {
                                    class: REMOVE_BTN,
                                    onclick: {
                                        let key = key.clone();
                                        let target = target.clone();
                                        move |_| {
                                            if let Some(parsed_key) = parse_id_or_defaults(&key) {
                                                let mut diagram = input_diagram.write();
                                                let Some(styles) = target.write_mut(&mut diagram) else {
                                                    return;
                                                };
                                                if let Some(partials) = styles.get_mut(&parsed_key)
                                                    && alias_idx < partials.style_aliases_applied.len()
                                                {
                                                    partials.style_aliases_applied.remove(alias_idx);
                                                }
                                            }
                                        }
                                    },
                                    "✕"
                                }
                            }
                        }
                    }
                }

                div {
                    class: ADD_BTN,
                    onclick: {
                        let key = entry_key.clone();
                        let target = target.clone();
                        move |_| {
                            if let Some(parsed_key) = parse_id_or_defaults(&key) {
                                let mut diagram = input_diagram.write();
                                let Some(styles) = target.write_mut(&mut diagram) else {
                                    return;
                                };
                                if let Some(partials) = styles.get_mut(&parsed_key) {
                                    // Default to `shade_light` as a sensible starting alias.
                                    partials
                                        .style_aliases_applied
                                        .push(StyleAlias::ShadeLight);
                                }
                            }
                        }
                    },
                    "+ Add alias"
                }
            }

            // ── Theme attributes (partials map) ──────────────────────
            div {
                class: "flex flex-col gap-1 pl-4",

                label {
                    class: LABEL_CLASS,
                    "Attributes"
                }

                for (attr_idx, (attr_name, attr_value)) in theme_attrs.iter().enumerate() {
                    {
                        let attr_name = attr_name.clone();
                        let attr_value = attr_value.clone();
                        let key = entry_key.clone();
                        let target = target.clone();
                        rsx! {
                            div {
                                key: "attr_{attr_idx}_{attr_name}",
                                class: ROW_CLASS_SIMPLE,

                                // Attribute name dropdown
                                select {
                                    class: SELECT_CLASS,
                                    value: "{attr_name}",
                                    onchange: {
                                        let key = key.clone();
                                        let old_attr_name = attr_name.clone();
                                        let current_value = attr_value.clone();
                                        let target = target.clone();
                                        move |evt: dioxus::events::FormEvent| {
                                            let new_attr_str = evt.value();
                                            if let (Some(old_attr), Some(new_attr)) = (
                                                parse_theme_attr(&old_attr_name),
                                                parse_theme_attr(&new_attr_str),
                                            )
                                                && old_attr != new_attr
                                                && let Some(parsed_key) = parse_id_or_defaults(&key)
                                            {
                                                let mut diagram = input_diagram.write();
                                                let Some(styles) = target.write_mut(&mut diagram) else {
                                                    return;
                                                };
                                                if let Some(partials) = styles.get_mut(&parsed_key) {
                                                    partials.partials.shift_remove(&old_attr);
                                                    partials.partials.insert(new_attr, current_value.clone());
                                                }
                                            }
                                        }
                                    },

                                    for (name, _) in THEME_ATTRS.iter() {
                                        option {
                                            value: "{name}",
                                            selected: *name == attr_name.as_str(),
                                            "{name}"
                                        }
                                    }
                                }

                                // Attribute value
                                input {
                                    class: INPUT_CLASS,
                                    style: "max-width:8rem",
                                    placeholder: "value",
                                    value: "{attr_value}",
                                    onchange: {
                                        let key = key.clone();
                                        let attr_name = attr_name.clone();
                                        let target = target.clone();
                                        move |evt: dioxus::events::FormEvent| {
                                            let new_val = evt.value();
                                            if let Some(attr) = parse_theme_attr(&attr_name)
                                                && let Some(parsed_key) = parse_id_or_defaults(&key)
                                            {
                                                let mut diagram = input_diagram.write();
                                                let Some(styles) = target.write_mut(&mut diagram) else {
                                                    return;
                                                };
                                                if let Some(partials) = styles.get_mut(&parsed_key)
                                                    && let Some(v) = partials.partials.get_mut(&attr)
                                                {
                                                    *v = new_val;
                                                }
                                            }
                                        }
                                    },
                                }

                                span {
                                    class: REMOVE_BTN,
                                    onclick: {
                                        let key = key.clone();
                                        let attr_name = attr_name.clone();
                                        let target = target.clone();
                                        move |_| {
                                            if let Some(attr) = parse_theme_attr(&attr_name)
                                                && let Some(parsed_key) = parse_id_or_defaults(&key)
                                            {
                                                let mut diagram = input_diagram.write();
                                                let Some(styles) = target.write_mut(&mut diagram) else {
                                                    return;
                                                };
                                                if let Some(partials) = styles.get_mut(&parsed_key) {
                                                    partials.partials.shift_remove(&attr);
                                                }
                                            }
                                        }
                                    },
                                    "✕"
                                }
                            }
                        }
                    }
                }

                div {
                    class: ADD_BTN,
                    onclick: {
                        let key = entry_key.clone();
                        let target = target.clone();
                        move |_| {
                            if let Some(parsed_key) = parse_id_or_defaults(&key) {
                                let mut diagram = input_diagram.write();
                                let Some(styles) = target.write_mut(&mut diagram) else {
                                    return;
                                };
                                if let Some(partials) = styles.get_mut(&parsed_key) {
                                    // Find first ThemeAttr not yet present.
                                    let new_attr = THEME_ATTRS
                                        .iter()
                                        .find(|(_, attr)| !partials.partials.contains_key(attr))
                                        .map(|(_, attr)| *attr);
                                    if let Some(attr) = new_attr {
                                        partials.partials.insert(attr, String::new());
                                    }
                                }
                            }
                        }
                    },
                    "+ Add attribute"
                }
            }
        }
    }
}
