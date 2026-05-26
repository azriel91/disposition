use disposition_ir_model::edge::EdgeId;
use disposition_model_common::Map;
use disposition_svg_model::{SvgEdgeDescriptionInfo, SvgTextSpan};
use disposition_taffy_model::{EdgeDescriptionTaffyNodes, EntityHighlightedSpan, TaffyNodeCtx};
use taffy::TaffyTree;

use crate::string_xml_escaper::StringXmlEscaper;

use super::svg_node_info_builder::SvgNodeInfoBuilder;

/// Builds [`SvgEdgeDescriptionInfo`] values from the edge description taffy
/// nodes and their computed highlighted spans.
#[derive(Clone, Copy, Debug)]
pub(super) struct SvgEdgeDescriptionsBuilder;

impl SvgEdgeDescriptionsBuilder {
    /// Returns a [`Vec`] of [`SvgEdgeDescriptionInfo`] for all edges that have
    /// description spans.
    pub(super) fn build<'id>(
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        edge_description_taffy_nodes: &Map<EdgeId<'id>, EdgeDescriptionTaffyNodes>,
        edge_description_highlighted_spans: &Map<EdgeId<'id>, Vec<EntityHighlightedSpan>>,
    ) -> Vec<SvgEdgeDescriptionInfo<'id>> {
        edge_description_taffy_nodes
            .iter()
            .filter_map(|(edge_id, edge_desc_taffy_nodes)| {
                let spans = edge_description_highlighted_spans.get(edge_id)?;
                if spans.is_empty() {
                    return None;
                }

                let description_taffy_node_id = edge_desc_taffy_nodes.description_taffy_node_id;
                let layout = taffy_tree.layout(description_taffy_node_id).ok()?;
                let (x, y) = SvgNodeInfoBuilder::node_absolute_xy_coordinates(
                    taffy_tree,
                    description_taffy_node_id,
                    layout,
                );

                let text_spans: Vec<SvgTextSpan> = spans
                    .iter()
                    .map(|span| {
                        SvgTextSpan::new(
                            x + span.x,
                            y + span.y,
                            StringXmlEscaper::escape(&span.text),
                        )
                    })
                    .collect();

                Some(SvgEdgeDescriptionInfo {
                    edge_id: edge_id.clone(),
                    x,
                    y,
                    width: layout.size.width,
                    height: layout.size.height,
                    text_spans,
                })
            })
            .collect()
    }
}
