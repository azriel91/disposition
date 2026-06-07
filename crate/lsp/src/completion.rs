//! Computes YAML key / value completions for an `InputDiagram` buffer.
//!
//! The [`CompletionEngine`] combines three sources:
//!
//! 1. [`CursorContext`] -- where the cursor is (the map-key path, and whether a
//!    key or value is being typed), resolved from line indentation.
//! 2. [`DiagramSchema`] -- the committed `InputDiagram` JSON schema, walked
//!    along the cursor path to find the valid keys / enum values.
//! 3. [`DynamicCompletions`] -- IDs already defined in the document, offered
//!    when a value position references an ID type.
//!
//! [`CompletionEngine`]: completion_engine::CompletionEngine
//! [`CursorContext`]: cursor_context::CursorContext
//! [`DiagramSchema`]: diagram_schema::DiagramSchema
//! [`DynamicCompletions`]: dynamic_completions::DynamicCompletions

pub mod completion_engine;
pub mod completion_target;
pub mod cursor_context;
pub mod diagram_schema;
pub mod dynamic_completions;
pub mod id_category;
pub mod key_category;
pub mod yaml_lines;

pub use self::completion_engine::CompletionEngine;
