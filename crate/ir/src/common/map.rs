//! `IndexMap` doesn't implement `utoipa::ToSchema`, and there doesn't yet exist
//! a nice way to implement the trait on third party types:
//!
//! <https://github.com/juhaku/utoipa/issues/790#issuecomment-1787754185>
//!
//! For now, we use a `HashMap` when the `openapi` feature is enabled.
//!
//! In general, this library should be built with the `"openapi"` feature
//! disabled.
//!
//! Tests rely on indexmap as some assertions expect order to be preserved.

#[cfg(all(feature = "openapi", not(test)))]
pub use std::collections::HashMap as Map;

#[cfg(any(not(feature = "openapi"), test))]
pub use indexmap::IndexMap as Map;
