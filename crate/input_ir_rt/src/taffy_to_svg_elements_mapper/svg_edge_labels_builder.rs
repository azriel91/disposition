use disposition_ir_model::edge::EdgeId;
use disposition_model_common::{Id, Map};
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
                // Spans are stored under `{edge_id}__from_label` and
                // `{edge_id}__to_label` keys, keyed separately so each
                // endpoint can display different text.
                let from_label_key = Id::try_from(format!("{edge_id}__from_label"))
                    .expect("`edge_id` is a valid `Id`, so appending `__from_label` is also valid");
                let to_label_key = Id::try_from(format!("{edge_id}__to_label"))
                    .expect("`edge_id` is a valid `Id`, so appending `__to_label` is also valid");

                let from_spans = entity_highlighted_spans.get(&from_label_key);
                let to_spans = entity_highlighted_spans.get(&to_label_key);

                let from_label =
                    edge_label_taffy_node_ids
                        .from_label_taffy_node_id
                        .and_then(|taffy_node_id| {
                            Self::endpoint_info_build(taffy_tree, taffy_node_id, from_spans)
                        });

                let to_label =
                    edge_label_taffy_node_ids
                        .to_label_taffy_node_id
                        .and_then(|taffy_node_id| {
                            Self::endpoint_info_build(taffy_tree, taffy_node_id, to_spans)
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
