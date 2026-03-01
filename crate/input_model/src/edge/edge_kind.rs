use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

/// Specifies the kind of edge.
///
/// Edges can be either cyclic (forming a loop), sequential (one-way chain),
/// or symmetric (forward then reverse).
///
/// # Examples
///
/// Valid string representations (snake_case):
///
/// * `"cyclic"`
/// * `"sequence"`
/// * `"symmetric"`
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    /// Last thing in the list has an edge back to first thing.
    ///
    /// Should have at least one `thing`. When there is only one thing,
    /// it represents a self-loop.
    Cyclic,

    /// A sequence of 2 or more things forming a one-way chain.
    ///
    /// The edge goes from the first thing to the second, second to third, etc.
    Sequence,

    /// A symmetric edge where things connect forward then back.
    ///
    /// For a list of things A, B, C, the edges are: A -> B -> C -> B -> A.
    /// Should have at least one `thing`. When there is only one thing,
    /// it represents a request and response to itself.
    Symmetric,
}

impl EdgeKind {
    /// Returns the string representation of this edge kind.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_input_model::edge::EdgeKind;
    ///
    /// assert_eq!(EdgeKind::Cyclic.as_str(), "cyclic");
    /// assert_eq!(EdgeKind::Sequence.as_str(), "sequence");
    /// assert_eq!(EdgeKind::Symmetric.as_str(), "symmetric");
    /// ```
    pub fn as_str(self) -> &'static str {
        match self {
            EdgeKind::Cyclic => "cyclic",
            EdgeKind::Sequence => "sequence",
            EdgeKind::Symmetric => "symmetric",
        }
    }
}

impl fmt::Display for EdgeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EdgeKind::Cyclic => write!(f, "cyclic"),
            EdgeKind::Sequence => write!(f, "sequence"),
            EdgeKind::Symmetric => write!(f, "symmetric"),
        }
    }
}

impl FromStr for EdgeKind {
    type Err = EdgeKindParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cyclic" => Ok(EdgeKind::Cyclic),
            "sequence" => Ok(EdgeKind::Sequence),
            "symmetric" => Ok(EdgeKind::Symmetric),
            _ => Err(EdgeKindParseError(s.to_owned())),
        }
    }
}

/// Error returned when parsing an invalid [`EdgeKind`] string.
///
/// # Examples
///
/// ```rust,should_panic
/// # use disposition_input_model::edge::EdgeKind;
/// let _: EdgeKind = "invalid".parse().unwrap();
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EdgeKindParseError(pub String);

impl fmt::Display for EdgeKindParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid edge kind `{}`, expected one of: `cyclic`, `sequence`, `symmetric`",
            self.0
        )
    }
}

impl std::error::Error for EdgeKindParseError {}
