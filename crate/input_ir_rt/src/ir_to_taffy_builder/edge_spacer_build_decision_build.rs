use disposition_ir_model::node::NodeId;

/// Parameters for building the edge spacer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EdgeSpacerBuildDecisionBuild<'f, 'id> {
    pub target_child_id: &'f NodeId<'id>,
}
