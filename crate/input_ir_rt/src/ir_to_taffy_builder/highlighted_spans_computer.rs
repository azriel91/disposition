use disposition_ir_model::edge::EdgeId;
use disposition_model_common::{
    edge::{EdgeDescs, EdgeGroupId},
    Map,
};
use disposition_taffy_model::{
    taffy::{self, TaffyTree},
    DiagramLod, EdgeDescriptionTaffyNodes, EntityHighlightedSpan, TaffyNodeCtx, TEXT_LINE_HEIGHT,
};

use super::text_measure::{line_width_measure, wrap_text_monospace};

/// Computes highlighted text spans for edge description containers after taffy
/// layout is complete.
///
/// Diagram node text and markdown edge descriptions are handled by
/// `MdSpansComputer`; this computer only covers the non-markdown edge
/// description container path.
pub(crate) struct HighlightedSpansComputer;

impl HighlightedSpansComputer {
    /// Computes highlighted spans for a single edge description leaf node.
    ///
    /// Returns `None` if `text` is empty or the taffy layout cannot be read.
    fn compute_edge_label_slot(
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        taffy_node_id: taffy::NodeId,
        text: &str,
        char_width: f32,
        line_height: f32,
    ) -> Option<Vec<EntityHighlightedSpan>> {
        if text.is_empty() {
            return None;
        }

        let Ok(node_layout) = taffy_tree.layout(taffy_node_id) else {
            return None;
        };

        let max_width = node_layout.size.width;
        let wrapped_lines = wrap_text_monospace(text, char_width, max_width);

        let padding_left = node_layout.padding.left;
        let padding_top = node_layout.padding.top;
        let text_leftmost_x = padding_left + 0.5 * char_width;

        let spans = wrapped_lines
            .iter()
            .enumerate()
            .map(|(line_index, line)| EntityHighlightedSpan {
                x: text_leftmost_x,
                y: (line_index + 1) as f32 * line_height + padding_top,
                width: line_width_measure(line, char_width),
                height: line_height,
                text: line.to_string(),
                md_style: None,
                tailwind_classes: Vec::new(),
            })
            .collect();

        Some(spans)
    }

    /// Computes highlighted spans for all edge description leaf nodes after
    /// layout is complete.
    ///
    /// Only runs at [`DiagramLod::Normal`]; returns an empty map at
    /// [`DiagramLod::Simple`].
    ///
    /// For each entry in `edge_description_taffy_nodes`, looks up the edge
    /// description text in `edge_descs`, reads the layout width of the
    /// `description_taffy_node_id` as the wrapping constraint, and builds
    /// [`EntityHighlightedSpan`] values relative to the leaf node's top-left
    /// corner.
    ///
    /// Edges with `md_node_taffy_ids.is_some()` (markdown path) are skipped;
    /// those are handled by `MdSpansComputer::compute_edge_descs`.
    pub(crate) fn compute_edge_desc_containers(
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        edge_description_taffy_nodes: &Map<EdgeId<'static>, EdgeDescriptionTaffyNodes>,
        edge_descs: &EdgeDescs<'static>,
        edge_id_to_group_id: &Map<EdgeId<'static>, EdgeGroupId<'static>>,
        char_width: f32,
        lod: &DiagramLod,
    ) -> Map<EdgeId<'static>, Vec<EntityHighlightedSpan>> {
        if !matches!(lod, DiagramLod::Normal) {
            return Map::new();
        }

        let line_height = TEXT_LINE_HEIGHT;

        edge_description_taffy_nodes
            .iter()
            .filter(|(_, edge_desc_taffy_nodes)| edge_desc_taffy_nodes.md_node_taffy_ids.is_none())
            .filter_map(|(edge_id, edge_desc_taffy_nodes)| {
                let edge_group_id = edge_id_to_group_id.get(edge_id)?;
                let desc = edge_descs.get_for_edge(edge_id, edge_group_id)?;
                let spans = Self::compute_edge_label_slot(
                    taffy_tree,
                    edge_desc_taffy_nodes.description_taffy_node_id,
                    desc,
                    char_width,
                    line_height,
                )?;
                Some((edge_id.clone(), spans))
            })
            .collect()
    }
}
