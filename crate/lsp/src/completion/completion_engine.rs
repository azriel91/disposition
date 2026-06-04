//! Computes completions at a cursor position from the schema + document IDs.

use async_lsp::lsp_types::{CompletionItem, CompletionItemKind};
use serde_json::Value;

use crate::completion::{
    completion_target::CompletionTarget, cursor_context::CursorContext,
    diagram_schema::DiagramSchema, dynamic_completions::DynamicCompletions, id_category::IdCategory,
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
        let container = schema.deref(container);

        match &cursor_context.target {
            CompletionTarget::Key => Self::key_completions(schema, container),
            CompletionTarget::Value { key } => {
                Self::value_completions(schema, container, key, text)
            }
        }
    }

    /// Offers the known fields of `container` as map-key completions.
    fn key_completions(schema: &DiagramSchema, container: &Value) -> Vec<CompletionItem> {
        schema
            .property_entries(container)
            .into_iter()
            .map(|property| CompletionItem {
                label: property.name.to_string(),
                kind: Some(CompletionItemKind::FIELD),
                detail: property.description.map(first_line),
                ..CompletionItem::default()
            })
            .collect()
    }

    /// Offers enum values and/or document-defined IDs for `key`'s value.
    fn value_completions(
        schema: &DiagramSchema,
        container: &Value,
        key: &str,
        text: &str,
    ) -> Vec<CompletionItem> {
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
        if let Some(category) = DiagramSchema::ref_name(element_schema)
            .and_then(IdCategory::from_ref_name)
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
