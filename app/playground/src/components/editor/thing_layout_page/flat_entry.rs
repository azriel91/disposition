//! Flat entry representation of the thing hierarchy.
//!
//! Re-exports types and functions from `disposition_input_rt::flat_entry` so
//! that existing callers within the playground continue to compile without
//! path changes.

pub use disposition_input_rt::flat_entry::{hierarchy_flatten, FlatEntry};
