use disposition_ir_model::{
    edge::EdgeGroups,
    entity::EntityTypes,
    layout::NodeLayouts,
    node::{
        NodeFaceEdges, NodeHierarchy, NodeId, NodeNames, NodeNestingInfos, NodeRanksNested,
        NodeShapes,
    },
    process::{ProcessStepGraphs, ProcessStepRanks},
};
use disposition_model_common::{edge::EdgeDescs, thing::ThingDescs, Id, Map};
use disposition_taffy_model::DiagramLod;

/// Immutable context shared across the taffy build functions.
///
/// Holds the read-only intermediate representation data and precomputed
/// values that nearly every taffy build function needs. Bundling them into a
/// single `Copy` value keeps the individual function signatures small,
/// avoiding the `#[allow(clippy::too_many_arguments)]` annotation.
///
/// Mutable accumulators (the taffy tree and the various output maps) live in
/// [`TaffyBuildState`] instead.
///
/// [`TaffyBuildState`]: super::taffy_build_state::TaffyBuildState
#[derive(Clone, Copy)]
pub(crate) struct TaffyBuildCtx<'ctx> {
    /// Layout (margin / padding / size) information for each node.
    pub(crate) node_layouts: &'ctx NodeLayouts<'static>,
    /// The node hierarchy currently being built.
    ///
    /// At the top level this is the diagram root's hierarchy; when recursing
    /// into a container node this is replaced with that container's child
    /// hierarchy via `TaffyBuildCtx { node_hierarchy, ..ctx }`.
    pub(crate) node_hierarchy: &'ctx NodeHierarchy<'static>,
    /// Entity types for each node, used to group nodes by entity type.
    pub(crate) entity_types: &'ctx EntityTypes<'static>,
    /// Descriptions for each thing (diagram node).
    pub(crate) thing_descs: &'ctx ThingDescs<'static>,
    /// Descriptions for each edge.
    pub(crate) edge_descs: &'ctx EdgeDescs<'static>,
    /// Shape (rect / circle) for each node.
    pub(crate) node_shapes: &'ctx NodeShapes<'static>,
    /// Ranks for nodes at each hierarchy level.
    pub(crate) node_ranks_nested: &'ctx NodeRanksNested<'static>,
    /// Ranks for process steps, derived from process step dependencies.
    ///
    /// Used to order process step taffy nodes within their process container.
    pub(crate) process_step_ranks: &'ctx ProcessStepRanks<'static>,
    /// Git-graph lane layout for each process's steps.
    ///
    /// Used to position process step circles into lane columns of a grid.
    pub(crate) process_step_graphs: &'ctx ProcessStepGraphs<'static>,
    /// Nesting information (ancestor chain / nesting path) for each node.
    pub(crate) node_nesting_infos: &'ctx NodeNestingInfos<'static>,
    /// Per-node face-to-edge-IDs mapping used to build envelope label slots.
    pub(crate) node_face_edges: &'ctx NodeFaceEdges<'static>,
    /// All edge groups in the diagram.
    pub(crate) edge_groups: &'ctx EdgeGroups<'static>,
    /// Level of detail for this diagram build.
    pub(crate) lod: DiagramLod,
    /// Monospace character width in pixels.
    pub(crate) char_width: f32,
    /// Precomputed markdown / text content for each diagram node.
    ///
    /// Computed once via [`Self::node_md_texts_compute`] so the text is
    /// consistent across the node-building, size-measuring, and
    /// highlighted-span passes.
    pub(crate) node_md_texts: &'ctx Map<NodeId<'static>, String>,
}

impl<'ctx> TaffyBuildCtx<'ctx> {
    /// Computes the markdown / text content for every diagram node.
    ///
    /// At [`DiagramLod::Simple`] the text is just the node name. At
    /// [`DiagramLod::Normal`] the node description (when present) is appended
    /// after a blank line, e.g. `"node name\n\ndescription"`.
    ///
    /// Entries are added for every named node, plus any thing that has a
    /// description but no name (keyed by its ID string), so that all three
    /// build passes look up the same precomputed string.
    pub(crate) fn node_md_texts_compute(
        nodes: &NodeNames<'static>,
        thing_descs: &ThingDescs<'static>,
        lod: DiagramLod,
    ) -> Map<NodeId<'static>, String> {
        let mut node_md_texts = Map::with_capacity(nodes.len());

        nodes.iter().for_each(|(node_id, node_name)| {
            let text = match lod {
                DiagramLod::Simple => node_name.clone(),
                DiagramLod::Normal => match thing_descs.get(node_id.as_ref()) {
                    Some(desc) => {
                        if node_name.is_empty() {
                            desc.clone()
                        } else {
                            format!("{node_name}\n\n{desc}")
                        }
                    }
                    None => node_name.clone(),
                },
            };
            node_md_texts.insert(node_id.clone(), text);
        });

        // Include things that have a description but no display name, so the
        // fallback to the ID string is never silently dropping a description.
        thing_descs.iter().for_each(|(id, desc)| {
            let node_id = NodeId::from(id.clone());
            if node_md_texts.contains_key(&node_id) {
                return;
            }
            let text = match lod {
                DiagramLod::Simple => id.as_str().to_string(),
                DiagramLod::Normal => format!("{id}\n\n{desc}"),
            };
            node_md_texts.insert(node_id, text);
        });

        node_md_texts
    }

    /// Returns the precomputed markdown / text content for the node with the
    /// given ID, if any.
    ///
    /// Returns `None` only for a node present in the hierarchy with neither a
    /// name nor a description; callers fall back to the node ID string in that
    /// case.
    pub(crate) fn node_md_text(&self, node_id: &Id<'static>) -> Option<&'ctx str> {
        let node_md_texts: &'ctx Map<NodeId<'static>, String> = self.node_md_texts;
        node_md_texts.get(node_id).map(String::as_str)
    }
}
