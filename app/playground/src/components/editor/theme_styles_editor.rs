//! Card-based editor for [`ThemeStyles`] maps.
//!
//! Instead of a raw YAML textarea, this module provides:
//!
//! - [`ThemeStylesEditor`]: iterates over a `ThemeStyles` map and renders one
//!   [`CssClassPartialsCard`] per entry, plus an "add" button.
//! - [`CssClassPartialsCard`]: renders the key (`IdOrDefaults`) as a `<select>`
//!   / text input, the `style_aliases_applied` list, and each `ThemeAttr ->
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

pub(crate) mod css_class_partials_card;
pub(crate) mod css_class_partials_card_aliases;
pub(crate) mod css_class_partials_card_attrs;
pub(crate) mod css_class_partials_card_header;
pub(crate) mod css_class_partials_snapshot;
pub(crate) mod theme_attr_entry;

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

use crate::components::editor::common::{parse_entity_type_id, parse_tag_id_or_defaults, ADD_BTN};

use self::{
    css_class_partials_card::CssClassPartialsCard,
    css_class_partials_snapshot::CssClassPartialsSnapshot, theme_attr_entry::ThemeAttrEntry,
};

// === Constants === //

/// All `ThemeAttr` variants paired with their `snake_case` serialisation name.
///
/// `ThemeAttr` derives `Serialize` with `#[serde(rename_all = "snake_case")]`
/// but exposes no iteration helper, so we maintain a static list.
pub(crate) const THEME_ATTRS: &[(&str, ThemeAttr)] = &[
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
///
/// Valid values: `"node_defaults"`, `"node_excluded_defaults"`,
/// `"edge_defaults"`.
pub(crate) const ID_OR_DEFAULTS_BUILTINS: &[(&str, &str)] = &[
    ("node_defaults", "Node Defaults"),
    ("node_excluded_defaults", "Node Excluded Defaults"),
    ("edge_defaults", "Edge Defaults"),
];

// === Helpers === //

/// Look up the `snake_case` name for a `ThemeAttr`.
fn theme_attr_name(attr: &ThemeAttr) -> &'static str {
    THEME_ATTRS
        .iter()
        .find(|(_, a)| a == attr)
        .map(|(name, _)| *name)
        .unwrap_or("unknown")
}

/// Parse a string into an `IdOrDefaults`.
///
/// Returns the matching built-in variant for `"node_defaults"`,
/// `"node_excluded_defaults"`, and `"edge_defaults"`, otherwise attempts to
/// parse as a custom `Id`.
///
/// Valid values: `"node_defaults"`, `"edge_defaults"`, `"app_server"`.
pub(crate) fn parse_id_or_defaults(s: &str) -> Option<IdOrDefaults<'static>> {
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
///
/// Valid values: `"fill_color"`, `"stroke_width"`, `"opacity"`.
pub(crate) fn parse_theme_attr(s: &str) -> Option<ThemeAttr> {
    THEME_ATTRS
        .iter()
        .find(|(name, _)| *name == s)
        .map(|(_, attr)| *attr)
}

// === ThemeStylesTarget === //

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
    /// `theme_types_styles[entity_type_key]` -- styles for a particular entity
    /// type.
    TypesStyles {
        /// The `EntityTypeId` key as a string (e.g. `"type_organisation"`).
        entity_type_key: String,
    },
    /// `theme_thing_dependencies_styles.things_included_styles`
    DependenciesIncluded,
    /// `theme_thing_dependencies_styles.things_excluded_styles`
    DependenciesExcluded,
    /// `theme_tag_things_focus[tag_key]` -- styles for a particular tag (or
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
    /// [`Self::TypesStyles`] or [`Self::TagFocus`] variants.
    pub(crate) fn read<'diag>(
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
    /// For [`Self::TypesStyles`] and [`Self::TagFocus`], the entry is inserted
    /// with a default value if it does not yet exist.
    pub(crate) fn write_mut<'diag>(
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
                Some(diagram.theme_types_styles.entry(type_id).or_default())
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
                Some(diagram.theme_tag_things_focus.entry(tag).or_default())
            }
        }
    }
}

// === ThemeStylesEditor === //

/// Card-based editor for a [`ThemeStyles`] map.
///
/// Renders one [`CssClassPartialsCard`] per `IdOrDefaults -> CssClassPartials`
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
    let entries: Vec<CssClassPartialsSnapshot> = theme_styles
        .iter()
        .map(
            |(key, css_partials): (&IdOrDefaults<'static>, &CssClassPartials<'static>)| {
                let entry_key = key.as_str().to_owned();
                let style_aliases_applied: Vec<String> = css_partials
                    .style_aliases_applied
                    .iter()
                    .map(|a: &StyleAlias<'static>| a.as_str().to_owned())
                    .collect();
                let theme_attrs: Vec<ThemeAttrEntry> = css_partials
                    .partials
                    .iter()
                    .map(|(attr, val): (&ThemeAttr, &String)| ThemeAttrEntry {
                        attr_name: theme_attr_name(attr).to_owned(),
                        attr_value: val.clone(),
                    })
                    .collect();
                CssClassPartialsSnapshot {
                    entry_key,
                    style_aliases_applied,
                    theme_attrs,
                }
            },
        )
        .collect();
    drop(diagram);

    rsx! {
        div {
            class: "flex flex-col gap-2",

            for (idx, entry) in entries.iter().enumerate() {
                {
                    let entry_key = entry.entry_key.clone();
                    let style_aliases = entry.style_aliases_applied.clone();
                    let theme_attrs = entry.theme_attrs.clone();
                    let target = target.clone();
                    rsx! {
                        CssClassPartialsCard {
                            key: "entry_{idx}_{entry_key}",
                            input_diagram,
                            target,
                            entry_index: idx,
                            entry_key,
                            style_aliases,
                            theme_attrs,
                        }
                    }
                }
            }

            // === Add entry button === //
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
