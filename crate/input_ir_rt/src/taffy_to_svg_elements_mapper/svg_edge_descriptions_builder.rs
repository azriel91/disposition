use disposition_ir_model::edge::EdgeId;
use disposition_model_common::Map;
use disposition_svg_model::{SvgEdgeDescriptionInfo, SvgImageSpan, SvgTextSpan};
use disposition_taffy_model::{
    EdgeDescriptionTaffyNodes, EntityHighlightedSpan, MdImageSpan, TaffyNodeCtx,
};
use taffy::TaffyTree;

use crate::string_xml_escaper::StringXmlEscaper;

use super::svg_node_info_builder::{svg_md_style_from, SvgNodeInfoBuilder};

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
        edge_description_image_spans: &Map<EdgeId<'id>, Vec<MdImageSpan>>,
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
                    .map(|span| SvgTextSpan {
                        x: x + span.x,
                        y: y + span.y,
                        width: span.width,
                        height: span.height,
                        text: StringXmlEscaper::escape(&span.text),
                        md_style: span.md_style.as_ref().map(svg_md_style_from),
                        tailwind_classes: span.tailwind_classes.clone(),
                    })
                    .collect();

                let image_spans: Vec<SvgImageSpan> = edge_description_image_spans
                    .get(edge_id)
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

                Some(SvgEdgeDescriptionInfo {
                    edge_id: edge_id.clone(),
                    x,
                    y,
                    width: layout.size.width,
                    height: layout.size.height,
                    text_spans,
                    image_spans,
                })
            })
            .collect()
    }
}
