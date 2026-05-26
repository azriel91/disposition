use std::borrow::Cow;

use disposition_ir_model::{
    edge::{EdgeId, EdgeLabels},
    node::{NodeId, NodeNames},
};
use disposition_model_common::{edge::EdgeDescs, thing::ThingDescs, Id, Map};
use disposition_taffy_model::{
    taffy::{self, TaffyTree},
    DiagramLod, EdgeDescriptionTaffyNodes, EdgeLabelTaffyNodeIds, EntityHighlightedSpan,
    EntityHighlightedSpans, NodeToTaffyNodeIds, TaffyNodeCtx, TEXT_LINE_HEIGHT,
};

use super::text_measure::{line_width_measure, wrap_text_monospace};

/// Computes highlighted text spans for diagram nodes, edge label slots, and
/// edge description containers after taffy layout is complete.
pub(crate) struct HighlightedSpansComputer;

impl HighlightedSpansComputer {
    /// Computes highlighted text spans for all entity nodes and edge label
    /// slots after taffy layout is complete.
    ///
    /// Runs once per layout pass instead of inside `measure()`, which may be
    /// called multiple times per node during layout computation.
    ///
    /// Nodes and edge label slots are sized by layout; this pass reads the
    /// computed widths to determine line-wrapping and span positions.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compute(
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        node_id_to_taffy: &Map<NodeId<'static>, NodeToTaffyNodeIds>,
        edge_label_taffy_nodes: &Map<EdgeId<'static>, EdgeLabelTaffyNodeIds>,
        nodes: &NodeNames<'static>,
        thing_descs: &ThingDescs<'static>,
        edge_labels: &EdgeLabels<'static>,
        char_width: f32,
        lod: &DiagramLod,
    ) -> EntityHighlightedSpans<'static> {
        let mut entity_highlighted_spans = EntityHighlightedSpans::with_capacity(
            node_id_to_taffy.len() + edge_label_taffy_nodes.len(),
        );

        let line_height = TEXT_LINE_HEIGHT;

        node_id_to_taffy
            .iter()
            .for_each(|(node_id, &taffy_node_ids)| {
                let (wrapper_node_layout, text_node_layout, diagram_node_ctx) = match taffy_node_ids
                {
                    NodeToTaffyNodeIds::Leaf { text_node_id } => {
                        let Ok(text_node_layout) = taffy_tree.layout(text_node_id) else {
                            return;
                        };
                        let Some(TaffyNodeCtx::DiagramNode(diagram_node_ctx)) =
                            taffy_tree.get_node_context(text_node_id)
                        else {
                            return;
                        };
                        (text_node_layout, text_node_layout, diagram_node_ctx)
                    }
                    NodeToTaffyNodeIds::Wrapper {
                        wrapper_node_id,
                        text_node_id,
                    }
                    | NodeToTaffyNodeIds::LeafWithCircle {
                        wrapper_node_id,
                        circle_node_id: _,
                        text_node_id,
                    }
                    | NodeToTaffyNodeIds::WrapperCircle {
                        wrapper_node_id,
                        label_wrapper_node_id: _,
                        circle_node_id: _,
                        text_node_id,
                    } => {
                        let Ok(wrapper_node_layout) = taffy_tree.layout(wrapper_node_id) else {
                            return;
                        };
                        let Ok(text_node_layout) = taffy_tree.layout(text_node_id) else {
                            return;
                        };
                        let Some(TaffyNodeCtx::DiagramNode(diagram_node_ctx)) =
                            taffy_tree.get_node_context(text_node_id)
                        else {
                            return;
                        };

                        (wrapper_node_layout, text_node_layout, diagram_node_ctx)
                    }
                };
                let text_label_offset = match taffy_node_ids {
                    NodeToTaffyNodeIds::Leaf { .. } | NodeToTaffyNodeIds::Wrapper { .. } => 0.0f32,
                    NodeToTaffyNodeIds::LeafWithCircle {
                        wrapper_node_id: _,
                        circle_node_id,
                        text_node_id: _,
                    }
                    | NodeToTaffyNodeIds::WrapperCircle {
                        wrapper_node_id: _,
                        label_wrapper_node_id: _,
                        circle_node_id,
                        text_node_id: _,
                    } => taffy_tree
                        .layout(circle_node_id)
                        .map(|circle_node_layout| {
                            // This could be:
                            //
                            // ```rust
                            // circle_node_layout.size.width + gap
                            // ```
                            //
                            // but we don't have the gap value
                            text_node_layout.location.x - circle_node_layout.location.x
                        })
                        .unwrap_or_default(),
                };

                let entity_id = &diagram_node_ctx.entity_id;

                // Build the text content
                let node_name = nodes
                    .get(entity_id)
                    .map(String::as_str)
                    .unwrap_or_else(|| entity_id.as_str());

                let text: Cow<'_, str> = match lod {
                    DiagramLod::Simple => Cow::Borrowed(node_name),
                    DiagramLod::Normal => {
                        let node_desc = thing_descs.get(entity_id).map(String::as_str);
                        match node_desc {
                            Some(desc) => Cow::Owned(format!("# {node_name}\n\n{desc}")),
                            None => Cow::Borrowed(node_name),
                        }
                    }
                };

                if text.is_empty() {
                    return;
                }

                // Use the computed layout width as constraint
                let max_width = text_node_layout.size.width;

                // Compute line wrapping using simple monospace calculation
                let wrapped_lines = wrap_text_monospace(&text, char_width, max_width);

                // Get style info for padding calculations
                let padding_left = text_node_layout.padding.left;
                let padding_top = wrapper_node_layout.padding.top;

                // Note: we shift the text by half a character width because even though we have
                // padding, the text still reaches the left and right edges of the node.
                //
                // The half a character width (at each end) is added to the node's width in
                // `line_width_measure`.
                let text_leftmost_x = text_label_offset + padding_left + 0.5 * char_width;

                let highlighted_spans: Vec<EntityHighlightedSpan> = {
                    wrapped_lines
                        .iter()
                        .enumerate()
                        .flat_map(|(line_index, line)| {
                            let x = text_leftmost_x;
                            let y = (line_index + 1) as f32 * line_height + padding_top;
                            let width = line_width_measure(line, char_width);

                            let entity_highlighted_span = EntityHighlightedSpan {
                                x,
                                y,
                                width,
                                height: line_height,
                                // style,
                                text: line.to_string(),
                            };

                            vec![entity_highlighted_span]
                        })
                        .collect()
                };

                entity_highlighted_spans.insert(node_id.as_ref().clone(), highlighted_spans);
            });

        // === Edge label spans === //
        //
        // For DiagramLod::Normal, compute highlighted spans for edge label
        // slots. The from_label slot uses `edge_label.from` as its text and
        // the to_label slot uses `edge_label.to`, allowing each endpoint to
        // show different text. Spans are stored under
        // `{edge_id}__from_label` and `{edge_id}__to_label` keys.
        if matches!(lod, DiagramLod::Normal) {
            edge_label_taffy_nodes
                .iter()
                .for_each(|(edge_id, edge_label_taffy_node_ids)| {
                    let Some(edge_label) = edge_labels.get(edge_id) else {
                        return;
                    };

                    // Compute and store highlighted spans for the from_label slot.
                    if let Some(from_taffy_node_id) =
                        edge_label_taffy_node_ids.from_label_taffy_node_id
                    {
                        let from_text = edge_label.from.as_str();
                        if let Some(from_spans) = Self::compute_edge_label_slot(
                            taffy_tree,
                            from_taffy_node_id,
                            from_text,
                            char_width,
                            line_height,
                        ) {
                            let from_label_key =
                                Id::try_from(format!("{edge_id}__from_label"))
                                    .expect("`edge_id` is a valid `Id`, so appending `__from_label` is also valid");
                            entity_highlighted_spans.insert(from_label_key, from_spans);
                        }
                    }

                    // Compute and store highlighted spans for the to_label slot.
                    if let Some(to_taffy_node_id) =
                        edge_label_taffy_node_ids.to_label_taffy_node_id
                    {
                        let to_text = edge_label.to.as_str();
                        if let Some(to_spans) = Self::compute_edge_label_slot(
                            taffy_tree,
                            to_taffy_node_id,
                            to_text,
                            char_width,
                            line_height,
                        ) {
                            let to_label_key =
                                Id::try_from(format!("{edge_id}__to_label"))
                                    .expect("`edge_id` is a valid `Id`, so appending `__to_label` is also valid");
                            entity_highlighted_spans.insert(to_label_key, to_spans);
                        }
                    }
                });
        }

        entity_highlighted_spans
    }

    /// Computes highlighted spans for a single edge label slot or edge
    /// description leaf node.
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
    pub(crate) fn compute_edge_desc_containers(
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        edge_description_taffy_nodes: &Map<EdgeId<'static>, EdgeDescriptionTaffyNodes>,
        edge_descs: &EdgeDescs<'static>,
        char_width: f32,
        lod: &DiagramLod,
    ) -> Map<EdgeId<'static>, Vec<EntityHighlightedSpan>> {
        if !matches!(lod, DiagramLod::Normal) {
            return Map::new();
        }

        let line_height = TEXT_LINE_HEIGHT;

        edge_description_taffy_nodes
            .iter()
            .filter_map(|(edge_id, edge_desc_taffy_nodes)| {
                let desc = edge_descs.get(edge_id.as_ref())?;
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
