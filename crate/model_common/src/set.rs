//! `IndexSet` doesn't implement `utoipa::ToSchema`, and there doesn't yet exist
//! a nice way to implement the trait on third party types:
//!
//! <https://github.com/juhaku/utoipa/issues/790#issuecomment-1787754185>
//!
//! For now, we use a `HashSet` when the `openapi` feature is enabled, or when
//! the `"test"` feature is enabled. This is because `TagThings` uses a
//! `Map<TagId, Set<ThingId>>`, and `utoipa` doesn't appear to support doubly
//! nested `ToSchema` types.
//!
//! This means in tests, we cannot test ordered sets with all features enabled.
//!
//! In general, this library should be built with the `"openapi"` feature
//! disabled.

#[cfg(any(feature = "openapi", feature = "test"))]
pub use std::collections::HashSet as Set;

#[cfg(all(not(feature = "openapi"), not(feature = "test")))]
pub use indexmap::IndexSet as Set;
