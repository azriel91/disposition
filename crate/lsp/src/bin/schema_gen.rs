//! Regenerates the committed `InputDiagram` JSON schema.
//!
//! The completion engine in `disposition_lsp` loads the schema from a committed
//! JSON file (`src/completion/input_diagram_schema.json`) rather than deriving
//! it at runtime -- this keeps `schemars` out of the wasm binary and avoids the
//! `IndexMap`/`OrderMap` feature conflict (the `JsonSchema` derive is disabled
//! under the model crates' `test` feature, which `--all-features` enables).
//!
//! Run this whenever the `InputDiagram` data model changes:
//!
//! ```bash
//! cargo run -p disposition_lsp --features schema-gen --bin schema_gen
//! ```
//!
//! It must be run *without* the `test` feature, and depends only on
//! `disposition_input_model` (not the `disposition` umbrella crate, which pulls
//! in `disposition_input_rt` -- that crate does not compile against `IndexMap`).

fn main() {
    #[cfg(all(feature = "schema-gen", not(feature = "test")))]
    {
        use std::path::Path;

        use disposition_input_model::InputDiagram;

        let mut schema = schemars::schema_for!(InputDiagram);
        theme_attr_inject(&mut schema);
        style_alias_casing_fix(&mut schema);

        let schema_json = serde_json::to_string_pretty(&schema)
            .expect("Failed to serialize `InputDiagram` JSON schema.");

        let out_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("completion")
            .join("input_diagram_schema.json");

        std::fs::create_dir_all(out_path.parent().expect("Expected parent directory."))
            .expect("Failed to create `completion` directory.");
        std::fs::write(&out_path, format!("{schema_json}\n"))
            .unwrap_or_else(|e| panic!("Failed to write schema to {}: {e}", out_path.display()));

        println!("Wrote schema to {}", out_path.display());
    }

    #[cfg(any(not(feature = "schema-gen"), feature = "test"))]
    eprintln!(
        "The `schema_gen` binary must be built with `--features schema-gen` and \
         without the `test` feature."
    );
}

/// Injects the `ThemeAttr` key enum into the generated schema.
///
/// `CssClassPartials.partials` is a `#[serde(flatten)]`ed `Map<ThemeAttr,
/// String>`, so `schemars` represents it as `additionalProperties: string` and
/// drops the `ThemeAttr` key enum entirely. To let the LSP offer the theme
/// attribute keys (`shape_color`, `stroke_style`, ..) as map-key completions,
/// add `ThemeAttr` to `$defs` and constrain `CssClassPartials`'s keys to it via
/// `propertyNames`.
#[cfg(all(feature = "schema-gen", not(feature = "test")))]
fn theme_attr_inject(schema: &mut schemars::Schema) {
    use disposition_input_model::theme::ThemeAttr;

    let mut theme_attr_value = schemars::schema_for!(ThemeAttr).to_value();
    if let Some(theme_attr_object) = theme_attr_value.as_object_mut() {
        // Document-level meta keys only belong on a root schema, not a `$defs`
        // entry.
        theme_attr_object.remove("$schema");
        theme_attr_object.remove("$id");
        theme_attr_object.remove("title");
    }

    let defs = schema
        .as_object_mut()
        .and_then(|root| root.get_mut("$defs"))
        .and_then(serde_json::Value::as_object_mut)
        .expect("Expected `$defs` in the `InputDiagram` schema.");

    defs.insert("ThemeAttr".to_string(), theme_attr_value);

    let css_class_partials = defs
        .get_mut("CssClassPartials")
        .and_then(serde_json::Value::as_object_mut)
        .expect("Expected `CssClassPartials` in the `InputDiagram` schema `$defs`.");
    css_class_partials.insert(
        "propertyNames".to_string(),
        serde_json::json!({ "$ref": "#/$defs/ThemeAttr" }),
    );
}

/// Corrects the `StyleAlias` schema to match its actual serialization.
///
/// `StyleAlias` has hand-written `Serialize` / `Deserialize` impls that emit
/// snake_case strings (e.g. `circle_xs`), but its `schemars` derive is unaware
/// of those and instead emits the Rust variant names as PascalCase `const`s
/// (e.g. `CircleXs`). The `Custom(Id)` variant is likewise emitted as an
/// externally-tagged object (`{ "Custom": <Id> }`) rather than the bare string
/// it actually deserializes from.
///
/// This rewrites each `oneOf` variant's `const` to snake_case, and replaces the
/// `Custom` object with a freeform string -- keeping the descriptions the derive
/// extracts from the doc comments.
#[cfg(all(feature = "schema-gen", not(feature = "test")))]
fn style_alias_casing_fix(schema: &mut schemars::Schema) {
    let style_alias_one_of = schema
        .as_object_mut()
        .and_then(|root| root.get_mut("$defs"))
        .and_then(serde_json::Value::as_object_mut)
        .and_then(|defs| defs.get_mut("StyleAlias"))
        .and_then(serde_json::Value::as_object_mut)
        .and_then(|style_alias| style_alias.get_mut("oneOf"))
        .and_then(serde_json::Value::as_array_mut)
        .expect("Expected `StyleAlias.oneOf` array in the `InputDiagram` schema `$defs`.");

    for variant in style_alias_one_of.iter_mut() {
        let Some(variant_object) = variant.as_object_mut() else {
            continue;
        };

        match variant_object.get("const").and_then(serde_json::Value::as_str) {
            Some(pascal_case) => {
                let snake_case = pascal_case_to_snake_case(pascal_case);
                variant_object.insert("const".to_string(), serde_json::Value::String(snake_case));
            }
            None => {
                // The `Custom(Id)` variant deserializes from any (non-well-known)
                // string, not the externally-tagged object the derive emits.
                let description = variant_object.get("description").cloned().unwrap_or_else(|| {
                    serde_json::Value::String("Custom user-defined style alias.".to_string())
                });
                variant_object.clear();
                variant_object.insert(
                    "type".to_string(),
                    serde_json::Value::String("string".to_string()),
                );
                variant_object.insert("description".to_string(), description);
            }
        }
    }
}

/// Converts a PascalCase identifier to snake_case.
///
/// A `_` is inserted before each uppercase letter (except the first), and before
/// a digit that follows a letter -- so `CircleXs` becomes `circle_xs` and
/// `Rounded2xl` becomes `rounded_2xl`, matching `StyleAlias::as_str`.
#[cfg(all(feature = "schema-gen", not(feature = "test")))]
fn pascal_case_to_snake_case(pascal_case: &str) -> String {
    let mut snake_case = String::with_capacity(pascal_case.len() + 4);
    let mut previous_is_alphabetic = false;

    for (char_index, character) in pascal_case.chars().enumerate() {
        let needs_separator = char_index > 0
            && (character.is_ascii_uppercase()
                || (character.is_ascii_digit() && previous_is_alphabetic));
        if needs_separator {
            snake_case.push('_');
        }
        snake_case.push(character.to_ascii_lowercase());
        previous_is_alphabetic = character.is_ascii_alphabetic();
    }

    snake_case
}
