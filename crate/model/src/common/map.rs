//! `IndexMap` doesn't implement `utoipa::ToSchema`, and there doesn't yet exist
//! a nice way to implement the trait on third party types:
//!
//! <https://github.com/juhaku/utoipa/issues/790#issuecomment-1787754185>
//!
//! For now, we use a `HashMap` when the `openapi` feature is enabled.
//!
//! In general, this library should be built with the `"openapi"` feature
//! disabled.

#[cfg(feature = "openapi")]
pub use std::collections::HashMap as Map;

#[cfg(not(feature = "openapi"))]
pub use indexmap::IndexMap as Map;
