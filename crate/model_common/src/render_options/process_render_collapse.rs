use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

/// Controls whether processes are rendered collapsed or expanded.
///
/// A collapsed process only shows its label, and expands to reveal its steps
/// when the process (or any of its steps) is focused. An expanded process
/// always shows all of its steps.
///
/// # Examples
///
/// ```rust
/// use disposition_model_common::ProcessRenderCollapse;
///
/// let expand_when_one = ProcessRenderCollapse::ExpandWhenOne;
/// let expand_always = ProcessRenderCollapse::ExpandAlways;
/// let collapse = ProcessRenderCollapse::Collapse;
/// ```
#[cfg_attr(
    all(feature = "schemars", not(feature = "test")),
    derive(schemars::JsonSchema)
)]
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessRenderCollapse {
    /// Processes are rendered expanded when there is only a single process in
    /// the diagram, and collapsed otherwise.
    #[default]
    ExpandWhenOne,
    /// Processes are always rendered fully expanded.
    ExpandAlways,
    /// Processes are rendered collapsed, expanding only when focused.
    Collapse,
}

impl ProcessRenderCollapse {
    /// Returns `true` if this is the default (`ExpandWhenOne`).
    pub fn is_default(&self) -> bool {
        matches!(self, ProcessRenderCollapse::ExpandWhenOne)
    }

    /// Returns whether processes should be rendered fully expanded, given the
    /// number of processes in the diagram.
    ///
    /// * `ExpandWhenOne`: expanded when there is at most one process.
    /// * `ExpandAlways`: always expanded.
    /// * `Collapse`: never expanded.
    pub fn process_render_expanded(&self, process_count: usize) -> bool {
        match self {
            ProcessRenderCollapse::ExpandWhenOne => process_count <= 1,
            ProcessRenderCollapse::ExpandAlways => true,
            ProcessRenderCollapse::Collapse => false,
        }
    }
}

impl FromStr for ProcessRenderCollapse {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "expand_when_one" => Ok(ProcessRenderCollapse::ExpandWhenOne),
            "expand_always" => Ok(ProcessRenderCollapse::ExpandAlways),
            "collapse" => Ok(ProcessRenderCollapse::Collapse),
            _ => Err(()),
        }
    }
}

impl Display for ProcessRenderCollapse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessRenderCollapse::ExpandWhenOne => write!(f, "expand_when_one"),
            ProcessRenderCollapse::ExpandAlways => write!(f, "expand_always"),
            ProcessRenderCollapse::Collapse => write!(f, "collapse"),
        }
    }
}
