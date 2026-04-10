//! `OrderMap 1.x` doesn't implement `schemars::JsonSchema`, pending:
//!
//! <https://github.com/GREsau/schemars/pull/516>
//!
//! For now, we use `IndexMap` when the `schemars` feature is enabled.
//!
//! In general, this library should be built with the `"schemars"` feature
//! disabled.
//!
//! Tests rely on OrderMap as some assertions expect order to be preserved.

#[cfg(all(feature = "schemars", not(feature = "test")))]
pub use indexmap::map::Keys;
#[cfg(all(feature = "schemars", not(feature = "test")))]
pub use indexmap::IndexMap as Map;

#[cfg(any(not(feature = "schemars"), feature = "test"))]
pub use ordermap::map::Keys;
#[cfg(any(not(feature = "schemars"), feature = "test"))]
pub use ordermap::OrderMap as Map;
