use disposition_ir_model::node::NodeId;
use disposition_model_common::Map;
use disposition_taffy_model::{
    taffy::TaffyTree, EntityHighlightedSpan, EntityHighlightedSpans, MdImageSpan, MdNodeTaffyIds,
    NodeToTaffyNodeIds, TaffyNodeCtx, TEXT_LINE_HEIGHT,
};

use crate::{AbsoluteCoordinates, TaffyNodeAbsoluteCoordinatesCalculator};

/// Computes `EntityHighlightedSpan` and `MdImageSpan` entries for nodes that
/// used the markdown content path.
pub(crate) struct MdSpansComputer;

impl MdSpansComputer {
    /// Computes highlighted text spans and image spans for all nodes that have
    /// `MdNodeTaffyIds` entries in `md_node_taffy_ids`.
    ///
    /// Runs after taffy layout is complete. For each markdown node the taffy
    /// tree is walked to collect the positions of every token and image leaf,
    /// which are then grouped into visual lines and merged into
    /// `EntityHighlightedSpan` and `MdImageSpan` values.
    ///
    /// Span coordinates are **node-relative** (relative to the diagram wrapper
    /// node's top-left corner), matching the convention used by
    /// `HighlightedSpansComputer`. The wrapper node is CSS-translated to its
    /// absolute position, so coordinates inside the `<g>` must not include that
    /// translation a second time.
    ///
    /// The `char_width` parameter is accepted for API consistency with the
    /// surrounding span-computation functions; token widths are read from the
    /// already-completed taffy layout rather than re-computed from
    /// `char_width`.
    pub(crate) fn compute(
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        node_id_to_taffy: &Map<NodeId<'static>, NodeToTaffyNodeIds>,
        md_node_taffy_ids: &Map<NodeId<'static>, MdNodeTaffyIds>,
        _char_width: f32,
    ) -> (
        EntityHighlightedSpans<'static>,
        Map<NodeId<'static>, Vec<MdImageSpan>>,
    ) {
        let mut entity_highlighted_spans =
            EntityHighlightedSpans::with_capacity(md_node_taffy_ids.len());
        let mut entity_image_spans: Map<NodeId<'static>, Vec<MdImageSpan>> = Map::new();

        for (node_id, md_node_taffy_ids_entry) in md_node_taffy_ids {
            // Compute the absolute position of the diagram wrapper node.
            //
            // The wrapper is CSS-translated to (wrapper_abs_x, wrapper_abs_y), so
            // all span coordinates must be relative to that origin to avoid being
            // double-translated when the `<g>` element is rendered.
            let wrapper_abs_xy = node_id_to_taffy
                .get(node_id)
                .and_then(|&taffy_node_ids| {
                    let wrapper_node_id = taffy_node_ids.wrapper_taffy_node_id();
                    let wrapper_layout = taffy_tree.layout(wrapper_node_id).ok()?;
                    Some(TaffyNodeAbsoluteCoordinatesCalculator::calculate(
                        taffy_tree,
                        wrapper_node_id,
                        wrapper_layout,
                    ))
                })
                .unwrap_or_default();

            let (highlighted_spans, image_spans) =
                Self::compute_node(taffy_tree, md_node_taffy_ids_entry, wrapper_abs_xy);

            if !highlighted_spans.is_empty() {
                entity_highlighted_spans.insert(node_id.as_ref().clone(), highlighted_spans);
            }

            if !image_spans.is_empty() {
                entity_image_spans.insert(node_id.clone(), image_spans);
            }
        }

        (entity_highlighted_spans, entity_image_spans)
    }

    /// Computes highlighted text spans and image spans for all edge
    /// descriptions that used the markdown content path.
    ///
    /// Runs after taffy layout is complete. For each edge with
    /// `md_node_taffy_ids.is_some()`, the taffy tree is walked to collect token
    /// and image leaf positions, which are grouped into visual lines and merged
    /// into `EntityHighlightedSpan` and `MdImageSpan` values.
    ///
    /// Span coordinates are **description-relative** (relative to the
    /// `description_taffy_node_id` top-left corner), matching the convention
    /// used by `HighlightedSpansComputer::compute_edge_desc_containers`.
    ///
    /// Returns two maps, keyed by `EdgeId`:
    /// - Text spans map: used by `SvgEdgeDescriptionsBuilder` to produce
    ///   `SvgTextSpan` values.
    /// - Image spans map: used by `SvgEdgeDescriptionsBuilder` to produce
    ///   `SvgImageSpan` values.
    pub(crate) fn compute_edge_descs(
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        edge_description_taffy_nodes: &Map<
            disposition_ir_model::edge::EdgeId<'static>,
            disposition_taffy_model::EdgeDescriptionTaffyNodes,
        >,
        _char_width: f32,
    ) -> (
        Map<disposition_ir_model::edge::EdgeId<'static>, Vec<EntityHighlightedSpan>>,
        Map<disposition_ir_model::edge::EdgeId<'static>, Vec<MdImageSpan>>,
    ) {
        let mut edge_description_highlighted_spans = Map::new();
        let mut edge_description_image_spans = Map::new();

        for (edge_id, edge_desc_taffy_nodes) in edge_description_taffy_nodes {
            let Some(md_node_taffy_ids) = &edge_desc_taffy_nodes.md_node_taffy_ids else {
                // Skip edges using the legacy single-leaf path.
                continue;
            };

            // Compute the absolute position of the description node.
            let description_abs_xy = taffy_tree
                .layout(edge_desc_taffy_nodes.description_taffy_node_id)
                .ok()
                .map(|layout| {
                    TaffyNodeAbsoluteCoordinatesCalculator::calculate(
                        taffy_tree,
                        edge_desc_taffy_nodes.description_taffy_node_id,
                        layout,
                    )
                })
                .unwrap_or_default();

            let (highlighted_spans, image_spans) =
                Self::compute_node(taffy_tree, md_node_taffy_ids, description_abs_xy);

            if !highlighted_spans.is_empty() {
                edge_description_highlighted_spans.insert(edge_id.clone(), highlighted_spans);
            }

            if !image_spans.is_empty() {
                edge_description_image_spans.insert(edge_id.clone(), image_spans);
            }
        }

        (
            edge_description_highlighted_spans,
            edge_description_image_spans,
        )
    }

    fn compute_node(
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        md_node_taffy_ids: &MdNodeTaffyIds,
        wrapper_abs_xy: AbsoluteCoordinates,
    ) -> (Vec<EntityHighlightedSpan>, Vec<MdImageSpan>) {
        let mut all_highlighted_spans = Vec::new();
        let mut all_image_spans = Vec::new();

        for block in &md_node_taffy_ids.block_taffy_ids {
            let mut pending: Vec<TokenPosition> = Vec::with_capacity(block.token_node_ids.len());

            for &taffy_node_id in &block.token_node_ids {
                let Ok(layout) = taffy_tree.layout(taffy_node_id) else {
                    continue;
                };
                let Some(ctx) = taffy_tree.get_node_context(taffy_node_id) else {
                    continue;
                };
                let AbsoluteCoordinates { x: abs_x, y: abs_y } =
                    TaffyNodeAbsoluteCoordinatesCalculator::calculate(
                        taffy_tree,
                        taffy_node_id,
                        layout,
                    );
                // Make coordinates relative to the wrapper node so they are not
                // double-translated by the CSS translate on the node's `<g>`.
                let rel_x = abs_x - wrapper_abs_xy.x;
                let rel_y = abs_y - wrapper_abs_xy.y;
                pending.push(TokenPosition {
                    abs_x: rel_x,
                    abs_y: rel_y,
                    layout_width: layout.size.width,
                    ctx: ctx.clone(),
                });
            }

            // Sort into visual lines: primary key = floor(rel_y), secondary key = rel_x.
            pending.sort_by(|a, b| {
                let line_a = a.abs_y.floor() as i32;
                let line_b = b.abs_y.floor() as i32;
                line_a.cmp(&line_b).then_with(|| {
                    a.abs_x
                        .partial_cmp(&b.abs_x)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            });

            Self::compute_node_block_spans(
                &pending,
                &mut all_highlighted_spans,
                &mut all_image_spans,
            );
        }

        (all_highlighted_spans, all_image_spans)
    }

    /// Processes a sorted slice of token positions for one block row, merging
    /// consecutive same-style `MdToken` entries into `EntityHighlightedSpan`
    /// values and converting `MdImage` entries into `MdImageSpan` values.
    fn compute_node_block_spans(
        pending: &[TokenPosition],
        highlighted_spans: &mut Vec<EntityHighlightedSpan>,
        image_spans: &mut Vec<MdImageSpan>,
    ) {
        let mut i = 0;
        while i < pending.len() {
            let line_y = pending[i].abs_y.floor() as i32;

            // Find the end index of this visual-line group.
            let line_end = {
                let mut end = i + 1;
                while end < pending.len() && pending[end].abs_y.floor() as i32 == line_y {
                    end += 1;
                }
                end
            };

            let line_items = &pending[i..line_end];
            let mut j = 0;
            while j < line_items.len() {
                let item = &line_items[j];
                match &item.ctx {
                    TaffyNodeCtx::MdToken(token_ctx) => {
                        let run_style = token_ctx.md_style.clone();
                        let run_start_x = item.abs_x;
                        let run_abs_y = item.abs_y;
                        let mut run_texts = vec![token_ctx.text.clone()];
                        // Track the right edge of the last token so the span
                        // width spans the full visual extent, including the
                        // inter-token gaps (the rendered word spaces). Summing
                        // token widths alone would omit those gaps.
                        let mut run_end_x = item.abs_x + item.layout_width;
                        j += 1;

                        // Collect consecutive tokens on the same line with the same style.
                        while j < line_items.len() {
                            if let TaffyNodeCtx::MdToken(next_ctx) = &line_items[j].ctx
                                && next_ctx.md_style == run_style
                            {
                                run_texts.push(next_ctx.text.clone());
                                run_end_x = line_items[j].abs_x + line_items[j].layout_width;
                                j += 1;
                                continue;
                            }
                            break;
                        }

                        let run_width = run_end_x - run_start_x;

                        let tailwind_classes = run_style.to_tailwind_classes();

                        highlighted_spans.push(EntityHighlightedSpan {
                            x: run_start_x,
                            y: run_abs_y + TEXT_LINE_HEIGHT,
                            width: run_width,
                            height: TEXT_LINE_HEIGHT,
                            text: run_texts.join(" "),
                            md_style: Some(run_style),
                            tailwind_classes,
                        });
                    }
                    TaffyNodeCtx::MdImage(image_ctx) => {
                        image_spans.push(MdImageSpan {
                            x: item.abs_x,
                            y: item.abs_y,
                            width: image_ctx.width,
                            height: image_ctx.height,
                            src: image_ctx.src.clone(),
                            alt: image_ctx.alt.clone(),
                        });
                        j += 1;
                    }
                    _ => {
                        // Non-markdown context nodes should not appear inside a markdown block
                        // row, but skip gracefully if encountered.
                        j += 1;
                    }
                }
            }

            i = line_end;
        }
    }
}

/// Position information for a single token or image in a markdown content
/// node, after taffy layout has been computed.
struct TokenPosition {
    /// X coordinate of the token's top-left corner, relative to the diagram
    /// wrapper node (matching the coordinate space used by legacy text spans).
    abs_x: f32,
    /// Y coordinate of the token's top-left corner, relative to the diagram
    /// wrapper node.
    abs_y: f32,
    /// Width of the token as computed by taffy layout.
    layout_width: f32,
    /// The taffy context for this leaf (either `MdToken` or `MdImage`).
    ctx: TaffyNodeCtx,
}
