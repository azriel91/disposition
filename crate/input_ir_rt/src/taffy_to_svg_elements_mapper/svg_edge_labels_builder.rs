use disposition_ir_model::{edge::EdgeRouteReversals, node::NodeId};
use disposition_model_common::Id;
use disposition_svg_model::{
    SvgEdgeLabelEndpointInfo, SvgEdgeLabelInfo, SvgImageSpan, SvgTextSpan,
};
use disposition_taffy_model::{
    EdgeIdToEdgeLabelTaffyNodeIds, EntityHighlightedSpans, NodeIdToImageSpans, TaffyNodeCtx,
};
use taffy::TaffyTree;

use crate::{
    string_xml_escaper::StringXmlEscaper, AbsoluteCoordinates,
    TaffyNodeAbsoluteCoordinatesCalculator,
};

use super::svg_node_info_builder::svg_md_style_from;

/// Builds [`SvgEdgeLabelInfo`] values from the edge label taffy nodes and
/// their computed markdown spans.
#[derive(Clone, Copy, Debug)]
pub(super) struct SvgEdgeLabelsBuilder;

impl SvgEdgeLabelsBuilder {
    /// Returns a [`Vec`] of [`SvgEdgeLabelInfo`] for all edges that have label
    /// slots.
    pub(super) fn build<'id>(
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        edge_label_taffy_nodes: &EdgeIdToEdgeLabelTaffyNodeIds<'id>,
        entity_highlighted_spans: &EntityHighlightedSpans<'id>,
        entity_image_spans: &NodeIdToImageSpans<'id>,
        edge_route_reversals: &EdgeRouteReversals<'id>,
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

                let from_label =
                    edge_label_taffy_node_ids
                        .from_label_taffy_node_id
                        .and_then(|taffy_node_id| {
                            Self::endpoint_info_build(
                                taffy_tree,
                                taffy_node_id,
                                &from_label_key,
                                entity_highlighted_spans,
                                entity_image_spans,
                            )
                        });

                let to_label =
                    edge_label_taffy_node_ids
                        .to_label_taffy_node_id
                        .and_then(|taffy_node_id| {
                            Self::endpoint_info_build(
                                taffy_tree,
                                taffy_node_id,
                                &to_label_key,
                                entity_highlighted_spans,
                                entity_image_spans,
                            )
                        });

                // Route-reversed edges are stored mirrored (`from`/`to`
                // swapped by `EdgeRouteNormalizer`, label text swapped to
                // match), so the mirror's from-slot data belongs to the
                // user-declared `to` endpoint. Swap the fields back so the
                // emitted `{edge_id}__from_label` group still contains the
                // user's `from` label at the real `from` node.
                let (from_label, to_label) = if edge_route_reversals.contains(edge_id) {
                    (to_label, from_label)
                } else {
                    (from_label, to_label)
                };

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
    /// The slot's text and image spans (computed via
    /// `MdSpansComputer::compute_edge_labels`) are looked up by `label_key`,
    /// and their slot-relative coordinates are offset by the slot's absolute
    /// position. Markdown styling (`md_style` / Tailwind classes) is preserved.
    ///
    /// Returns `None` if the layout cannot be read.
    fn endpoint_info_build<'id>(
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        taffy_node_id: taffy::NodeId,
        label_key: &Id<'id>,
        entity_highlighted_spans: &EntityHighlightedSpans<'id>,
        entity_image_spans: &NodeIdToImageSpans<'id>,
    ) -> Option<SvgEdgeLabelEndpointInfo> {
        let layout = taffy_tree.layout(taffy_node_id).ok()?;
        let AbsoluteCoordinates { x, y } =
            TaffyNodeAbsoluteCoordinatesCalculator::calculate(taffy_tree, taffy_node_id, layout);
        let width = layout.size.width;
        let height = layout.size.height;

        let text_spans: Vec<SvgTextSpan> = entity_highlighted_spans
            .get(label_key)
            .map(|span_list| {
                span_list
                    .iter()
                    .map(|span| SvgTextSpan {
                        x: x + span.x,
                        y: y + span.y,
                        width: span.width,
                        height: span.height,
                        text: StringXmlEscaper::escape(&span.text),
                        md_style: span.md_style.as_ref().map(svg_md_style_from),
                        tailwind_classes: span.tailwind_classes.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let image_key: NodeId<'id> = label_key.clone().into();
        let image_spans: Vec<SvgImageSpan> = entity_image_spans
            .get(&image_key)
            .map(|img_spans| {
                img_spans
                    .iter()
                    .map(|span| SvgImageSpan {
                        x: x + span.x,
                        y: y + span.y,
                        width: span.width,
                        height: span.height,
                        src: span.src.clone(),
                        alt: span.alt.clone(),
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
            image_spans,
        })
    }
}
