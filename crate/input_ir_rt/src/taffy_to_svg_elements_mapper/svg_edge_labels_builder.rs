use disposition_ir_model::edge::EdgeId;
use disposition_model_common::Map;
use disposition_svg_model::{SvgEdgeLabelEndpointInfo, SvgEdgeLabelInfo, SvgTextSpan};
use disposition_taffy_model::{EdgeLabelTaffyNodeIds, EntityHighlightedSpans, TaffyNodeCtx};
use taffy::TaffyTree;

use crate::string_xml_escaper::StringXmlEscaper;

use super::svg_node_info_builder::SvgNodeInfoBuilder;

/// Builds [`SvgEdgeLabelInfo`] values from the edge label taffy nodes and
/// their computed highlighted spans.
#[derive(Clone, Copy, Debug)]
pub(super) struct SvgEdgeLabelsBuilder;

impl SvgEdgeLabelsBuilder {
    /// Returns a [`Vec`] of [`SvgEdgeLabelInfo`] for all edges that have label
    /// slots.
    pub(super) fn build<'id>(
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        edge_label_taffy_nodes: &Map<EdgeId<'id>, EdgeLabelTaffyNodeIds>,
        entity_highlighted_spans: &EntityHighlightedSpans<'id>,
    ) -> Vec<SvgEdgeLabelInfo<'id>> {
        edge_label_taffy_nodes
            .iter()
            .map(|(edge_id, edge_label_taffy_node_ids)| {
                let spans = entity_highlighted_spans.get(edge_id.as_ref());

                let from_label =
                    edge_label_taffy_node_ids
                        .from_label_taffy_node_id
                        .and_then(|taffy_node_id| {
                            Self::endpoint_info_build(taffy_tree, taffy_node_id, spans)
                        });

                let to_label =
                    edge_label_taffy_node_ids
                        .to_label_taffy_node_id
                        .and_then(|taffy_node_id| {
                            Self::endpoint_info_build(taffy_tree, taffy_node_id, spans)
                        });

                SvgEdgeLabelInfo {
                    edge_id: edge_id.clone(),
                    from_label,
                    to_label,
                }
            })
            .collect()
    }

    /// Builds a [`SvgEdgeLabelEndpointInfo`] for a single label taffy node.
    ///
    /// Returns `None` if the layout cannot be read.
    fn endpoint_info_build(
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        taffy_node_id: taffy::NodeId,
        spans: Option<&Vec<disposition_taffy_model::EntityHighlightedSpan>>,
    ) -> Option<SvgEdgeLabelEndpointInfo> {
        let layout = taffy_tree.layout(taffy_node_id).ok()?;
        let (x, y) =
            SvgNodeInfoBuilder::node_absolute_xy_coordinates(taffy_tree, taffy_node_id, layout);
        let width = layout.size.width;
        let height = layout.size.height;

        let text_spans: Vec<SvgTextSpan> = spans
            .map(|span_list| {
                span_list
                    .iter()
                    .map(|span| {
                        SvgTextSpan::new(
                            x + span.x,
                            y + span.y,
                            StringXmlEscaper::escape(&span.text),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();

        Some(SvgEdgeLabelEndpointInfo {
            x,
            y,
            width,
            height,
            text_spans,
        })
    }
}
