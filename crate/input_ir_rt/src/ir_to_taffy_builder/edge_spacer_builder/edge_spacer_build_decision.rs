use crate::ir_to_taffy_builder::edge_spacer_builder::{
    EdgeSpacerBuildDecisionBuild, EdgeSpacerBuildDecisionSkip,
};

/// Represents the decision to build or skip the edge spacer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EdgeSpacerBuildDecision<'f, 'id> {
    /// Skip building the edge spacer and use the default spacing instead.
    Skip(EdgeSpacerBuildDecisionSkip<'id>),
    /// Build the edge spacer using the specified spacing.
    Build(EdgeSpacerBuildDecisionBuild<'f, 'id>),
}
