//! Computes completions at a cursor position from the schema + document IDs.

use std::collections::BTreeSet;

use async_lsp::lsp_types::{CompletionItem, CompletionItemKind};
use disposition_input_model::theme::ThemeAttr;
use serde_json::Value;

use crate::completion::{
    completion_target::CompletionTarget, cursor_context::CursorContext,
    diagram_schema::DiagramSchema, dynamic_completions::DynamicCompletions,
    id_category::IdCategory, key_category::KeyCategory,
};

/// Produces YAML key / value completions for an `InputDiagram` buffer.
pub struct CompletionEngine;

impl CompletionEngine {
    /// Returns the completion items for the cursor at `line` / `character`
    /// (zero-based) within `text`.
    pub fn completions(text: &str, line: u32, character: u32) -> Vec<CompletionItem> {
        let schema = DiagramSchema::get();
        let cursor_context = CursorContext::at(text, line, character);

        let Some(container) = schema.schema_at(&cursor_context.path) else {
            return Vec::new();
        };
        // The def name (`$ref`) of the container *before* dereferencing -- e.g.
        // `ThingNames` -- identifies which dynamic map-key suggestions to offer.
        let container_ref_name = DiagramSchema::ref_name(container);
        let container = schema.deref(container);

        match &cursor_context.target {
            CompletionTarget::Key => Self::key_completions(
                schema,
                container,
                container_ref_name,
                &cursor_context.sibling_keys,
                text,
            ),
            CompletionTarget::Value { key } => {
                Self::value_completions(schema, container, container_ref_name, key, text)
            }
        }
    }

    /// Offers the known fields of `container` as map-key completions.
    ///
    /// In addition to the struct fields (`properties`), this offers the keys an
    /// arbitrary-map container constrains its entries to via `propertyNames`
    /// (e.g. the `ThemeAttr` keys of a `CssClassPartials` map -- `shape_color`,
    /// `stroke_style`, ..).
    fn key_completions(
        schema: &DiagramSchema,
        container: &Value,
        container_ref_name: Option<&str>,
        sibling_keys: &BTreeSet<String>,
        text: &str,
    ) -> Vec<CompletionItem> {
        let mut items = schema
            .property_entries(container)
            .into_iter()
            .map(|property| CompletionItem {
                label: property.name.to_string(),
                kind: Some(CompletionItemKind::FIELD),
                detail: property.description.map(first_line),
                ..CompletionItem::default()
            })
            .collect::<Vec<CompletionItem>>();

        // Keys constrained to an enum, e.g. a `Map<ThemeAttr, _>`'s theme
        // attribute keys.
        if let Some(property_names) = container.get("propertyNames") {
            items.extend(
                schema
                    .enum_entries(property_names)
                    .into_iter()
                    .map(|entry| CompletionItem {
                        label: entry.value.to_string(),
                        kind: Some(CompletionItemKind::FIELD),
                        detail: entry.description.map(first_line),
                        ..CompletionItem::default()
                    }),
            );
        }

        // Dynamic keys for maps whose keys are document-defined IDs, ID
        // templates, or known literal keys (e.g. a `ThingNames` map keyed by
        // `ThingId`).
        if let Some(key_category) = container_ref_name.and_then(KeyCategory::from_ref_name) {
            items.extend(Self::dynamic_key_completions(schema, key_category, text));
        }

        // A map key can only be declared once, so drop any already present as a
        // sibling of the cursor.
        items.retain(|item| !sibling_keys.contains(&item.label));

        items
    }

    /// Offers the dynamic map-key suggestions for `key_category`.
    ///
    /// Combines the document-derived / templated / literal labels from
    /// [`DynamicCompletions`] with any schema-derived built-in keys (the
    /// `StyleAlias` / `EntityType` enum values).
    fn dynamic_key_completions(
        schema: &DiagramSchema,
        key_category: KeyCategory,
        text: &str,
    ) -> Vec<CompletionItem> {
        let dynamic_completions = DynamicCompletions::from_text(text);

        let mut items = dynamic_completions
            .key_suggestions(key_category)
            .into_iter()
            .map(|label| CompletionItem {
                label,
                kind: Some(CompletionItemKind::VALUE),
                ..CompletionItem::default()
            })
            .collect::<Vec<CompletionItem>>();

        // Built-in enum keys defined in the schema.
        let builtin_def_name = match key_category {
            KeyCategory::StyleAlias => Some("StyleAlias"),
            KeyCategory::EntityType => Some("EntityType"),
            _ => None,
        };
        if let Some(def) = builtin_def_name.and_then(|name| schema.def(name)) {
            items.extend(
                schema
                    .enum_entries(def)
                    .into_iter()
                    .map(|entry| CompletionItem {
                        label: entry.value.to_string(),
                        kind: Some(CompletionItemKind::ENUM_MEMBER),
                        detail: entry.description.map(first_line),
                        ..CompletionItem::default()
                    }),
            );
        }

        items
    }

    /// Offers enum values and/or document-defined IDs for `key`'s value.
    fn value_completions(
        schema: &DiagramSchema,
        container: &Value,
        container_ref_name: Option<&str>,
        key: &str,
        text: &str,
    ) -> Vec<CompletionItem> {
        // `CssClassPartials` values keyed by a `ThemeAttr` are partial Tailwind
        // values whose vocabulary (colors, shades, styles, ..) depends on the
        // attribute and is not expressible in the JSON schema. The `key` may
        // instead be the `style_aliases_applied` property, which falls through
        // to the normal schema-driven completion below.
        if container_ref_name == Some("CssClassPartials")
            && let Some(items) = Self::theme_attr_value_completions(key)
        {
            return items;
        }

        let Some(value_schema) = schema.field_schema(container, key) else {
            return Vec::new();
        };

        // For an array-valued field (e.g. `things`), complete its element type.
        let element_schema = schema.array_items(value_schema).unwrap_or(value_schema);

        let mut items = Vec::new();

        // Fixed enum values (e.g. `row`, `cyclic`, `top_to_bottom`).
        items.extend(
            schema
                .enum_entries(element_schema)
                .into_iter()
                .map(|entry| CompletionItem {
                    label: entry.value.to_string(),
                    kind: Some(CompletionItemKind::ENUM_MEMBER),
                    detail: entry.description.map(first_line),
                    ..CompletionItem::default()
                }),
        );

        // Document-defined IDs, when the value references an ID type.
        if let Some(category) =
            DiagramSchema::ref_name(element_schema).and_then(IdCategory::from_ref_name)
        {
            let dynamic_completions = DynamicCompletions::from_text(text);
            items.extend(dynamic_completions.ids_for(category).into_iter().map(|id| {
                CompletionItem {
                    label: id.to_string(),
                    kind: Some(CompletionItemKind::VALUE),
                    ..CompletionItem::default()
                }
            }));
        }

        items
    }

    /// Offers the partial Tailwind values for a `CssClassPartials` value keyed
    /// by `key`, if `key` is a `ThemeAttr`.
    ///
    /// Returns `None` when `key` is not a theme attribute (e.g. the
    /// `style_aliases_applied` property), so the caller can fall back to the
    /// schema-driven completion. A theme attribute with no enumerable values
    /// (numeric / freeform, e.g. `padding`) yields `Some(<empty>)`.
    fn theme_attr_value_completions(key: &str) -> Option<Vec<CompletionItem>> {
        let theme_attr = serde_json::from_value::<ThemeAttr>(Value::String(key.to_string())).ok()?;

        let items = theme_attr
            .value_suggestions()
            .iter()
            .map(|value| CompletionItem {
                label: (*value).to_string(),
                kind: Some(CompletionItemKind::VALUE),
                ..CompletionItem::default()
            })
            .collect();

        Some(items)
    }
}

/// Returns the first non-empty line of `description`, trimmed -- used as the
/// short completion detail.
fn first_line(description: &str) -> String {
    description
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or_default()
        .to_string()
}
