//! `OrderSet 1.x` doesn't implement `schemars::JsonSchema`, pending:
//!
//! <https://github.com/GREsau/schemars/pull/516>
//!
//! For now, we use an `IndexSet` when the `schemars` feature is enabled.
//!
//! In general, this library should be built with the `"schemars"` feature
//! disabled.
//!
//! Tests rely on `ordermap::Set` as some assertions expect order to be
//! preserved.

#[cfg(all(feature = "schemars", not(feature = "test")))]
pub use indexmap::IndexSet as Set;

#[cfg(any(not(feature = "schemars"), feature = "test"))]
pub use ordermap::OrderSet as Set;
