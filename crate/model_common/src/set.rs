//! `IndexSet` doesn't implement `utoipa::ToSchema`, and there doesn't yet exist
//! a nice way to implement the trait on third party types:
//!
//! <https://github.com/juhaku/utoipa/issues/790#issuecomment-1787754185>
//!
//! For now, we use a `HashSet` when the `openapi` feature is enabled.
//!
//! In general, this library should be built with the `"openapi"` feature
//! disabled.
//!
//! Tests rely on indexmap::Set as some assertions expect order to be preserved.
//!
//! ⚠️ Note: when the `"test"` feature is enabled, even though the `"openapi"`
//! feature is enabled, we still disable `utoipa` because `utoipa` doesn't
//! support doubly nested `ToSchema` types (i.e. `Map<TagId, Set<ThingId>>`,
//! which `TagThings` is).

#[cfg(all(feature = "openapi", not(feature = "test")))]
pub use std::collections::HashSet as Set;

#[cfg(any(not(feature = "openapi"), feature = "test"))]
pub use indexmap::IndexSet as Set;
