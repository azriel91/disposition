use disposition_ir_model::node::NodeId;
use disposition_model_common::Map;
use disposition_taffy_model::{
    taffy::{self, TaffyTree},
    EntityHighlightedSpan, EntityHighlightedSpans, MdHeadingLevel, MdImageSpan, MdNodeTaffyIds,
    TaffyNodeCtx, TEXT_LINE_HEIGHT,
};

/// Computes `EntityHighlightedSpan` and `MdImageSpan` entries for nodes that
/// used the markdown content path.
pub(crate) struct MdSpansComputer;

impl MdSpansComputer {
    /// Computes highlighted text spans and image spans for all nodes that have
    /// `MdNodeTaffyIds` entries in `md_node_taffy_ids`.
    ///
    /// Runs after taffy layout is complete. For each markdown node the taffy
    /// tree is walked to collect the absolute positions of every token and
    /// image leaf, which are then grouped into visual lines and merged into
    /// `EntityHighlightedSpan` and `MdImageSpan` values.
    ///
    /// The `char_width` parameter is accepted for API consistency with the
    /// surrounding span-computation functions; token widths are read from the
    /// already-completed taffy layout rather than re-computed from
    /// `char_width`.
    pub(crate) fn compute(
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
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
            let (highlighted_spans, image_spans) =
                Self::compute_node(taffy_tree, md_node_taffy_ids_entry);

            if !highlighted_spans.is_empty() {
                entity_highlighted_spans.insert(node_id.as_ref().clone(), highlighted_spans);
            }

            if !image_spans.is_empty() {
                entity_image_spans.insert(node_id.clone(), image_spans);
            }
        }

        (entity_highlighted_spans, entity_image_spans)
    }

    fn compute_node(
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        md_node_taffy_ids: &MdNodeTaffyIds,
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
                let (abs_x, abs_y) =
                    Self::node_absolute_xy_coordinates(taffy_tree, taffy_node_id, layout);
                pending.push(TokenPosition {
                    abs_x,
                    abs_y,
                    layout_width: layout.size.width,
                    ctx: ctx.clone(),
                });
            }

            // Sort into visual lines: primary key = floor(abs_y), secondary key = abs_x.
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
                        let mut run_width = item.layout_width;
                        j += 1;

                        // Collect consecutive tokens on the same line with the same style.
                        while j < line_items.len() {
                            if let TaffyNodeCtx::MdToken(next_ctx) = &line_items[j].ctx {
                                if next_ctx.md_style == run_style {
                                    run_texts.push(next_ctx.text.clone());
                                    run_width += line_items[j].layout_width;
                                    j += 1;
                                    continue;
                                }
                            }
                            break;
                        }

                        let font_scale = run_style
                            .heading_level
                            .map(MdHeadingLevel::font_scale)
                            .unwrap_or(1.0);
                        let effective_line_height = TEXT_LINE_HEIGHT * font_scale;

                        highlighted_spans.push(EntityHighlightedSpan {
                            x: run_start_x,
                            y: run_abs_y + effective_line_height,
                            width: run_width,
                            height: effective_line_height,
                            text: run_texts.join(" "),
                            md_style: Some(run_style),
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

    /// Calculates the absolute x and y coordinates of a node by traversing
    /// up the parent chain and accumulating position offsets.
    ///
    /// This mirrors `SvgNodeInfoBuilder::node_absolute_xy_coordinates` but
    /// operates within `MdSpansComputer` to avoid a cross-module visibility
    /// dependency.
    fn node_absolute_xy_coordinates(
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        taffy_node_id: taffy::NodeId,
        layout: &taffy::Layout,
    ) -> (f32, f32) {
        let mut x_acc = layout.location.x;
        let mut y_acc = layout.location.y;
        let mut current_node_id = taffy_node_id;
        while let Some(parent_taffy_node_id) = taffy_tree.parent(current_node_id) {
            let Ok(parent_layout) = taffy_tree.layout(parent_taffy_node_id) else {
                break;
            };
            x_acc += parent_layout.location.x;
            y_acc += parent_layout.location.y;
            current_node_id = parent_taffy_node_id;
        }
        (x_acc, y_acc)
    }
}

/// Position information for a single token or image in a markdown content
/// node, after taffy layout has been computed.
struct TokenPosition {
    /// Absolute x coordinate of the token's top-left corner in the SVG space.
    abs_x: f32,
    /// Absolute y coordinate of the token's top-left corner in the SVG space.
    abs_y: f32,
    /// Width of the token as computed by taffy layout.
    layout_width: f32,
    /// The taffy context for this leaf (either `MdToken` or `MdImage`).
    ctx: TaffyNodeCtx,
}
