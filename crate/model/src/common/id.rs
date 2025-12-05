use std::borrow::Cow;

use serde::{Deserialize, Serialize};

/// Unique identifier for any entity in the diagram, `Cow<'static, str>`
/// newtype.
///
/// Must begin with a letter or underscore, and contain only letters, numbers,
/// and underscores.
///
/// # Examples
///
/// ```rust
/// use disposition_model::common::{id, Id};
///
/// let id_compile_time_checked = id!("example_id");
/// let id_runtime_checked = Id::new("example_id").unwrap();
///
/// assert_eq!(id_compile_time_checked, id_runtime_checked);
/// assert_eq!(id_runtime_checked.as_str(), "example_id");
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct Id(Cow<'static, str>);

id_newtype::id_newtype!(Id, IdInvalidFmt, id);
