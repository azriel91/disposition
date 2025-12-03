use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::common::{Id, IdInvalidFmt};

/// Unique identifier for a style alias, [`Id`] newtype.
///
/// Style aliases allow grouping of style properties under a single name,
/// which can then be applied to nodes and edges.
///
/// Must begin with a letter or underscore, and contain only letters, numbers,
/// and underscores.
///
/// # Examples
///
/// ```rust
/// use disposition_model::{
///     common::{id, Id},
///     theme::StyleAliasId,
/// };
///
/// let style_alias_id: StyleAliasId = id!("padding_normal").into();
///
/// assert_eq!(style_alias_id.as_str(), "padding_normal");
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct StyleAliasId(Id);

impl StyleAliasId {
    /// Creates a new [`StyleAliasId`] from a string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_model::{common::Id, theme::StyleAliasId};
    ///
    /// let style_alias_id = StyleAliasId::new("padding_normal").unwrap();
    ///
    /// assert_eq!(style_alias_id.as_str(), "padding_normal");
    /// ```
    pub fn new(id: &'static str) -> Result<Self, IdInvalidFmt<'static>> {
        Id::new(id).map(StyleAliasId)
    }

    /// Returns the underlying [`Id`] value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_model::{common::Id, theme::StyleAliasId};
    ///
    /// let style_alias_id = StyleAliasId::new("padding_normal").unwrap();
    ///
    /// assert_eq!(
    ///     style_alias_id.into_inner(),
    ///     Id::new("padding_normal").unwrap()
    /// );
    /// ```
    pub fn into_inner(self) -> Id {
        self.0
    }
}

impl From<Id> for StyleAliasId {
    fn from(id: Id) -> Self {
        StyleAliasId(id)
    }
}

impl Deref for StyleAliasId {
    type Target = Id;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for StyleAliasId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
