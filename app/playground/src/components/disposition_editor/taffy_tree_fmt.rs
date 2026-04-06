//! Taffy tree formatting helpers.
//!
//! Provides [`TaffyTreeFmt`] which writes a human-readable string
//! representation of a Taffy layout tree, annotated with disposition
//! diagram node IDs.

use std::{borrow::Cow, fmt::Write};

use disposition::{
    ir_model::node::{NodeId, NodeInbuilt},
    model_common::Map,
    taffy_model::{
        taffy::{self, PrintTree, TaffyTree, TraversePartialTree},
        TaffyNodeCtx, TaffyNodeMappings,
    },
};

/// Taffy tree formatting utilities.
///
/// These functions are copied and modified from the
/// `taffy::TaffyTree::print_tree` method:
///
/// <https://github.com/DioxusLabs/taffy/blob/v0.9.2/src/util/print.rs#L5>
///
/// then adapted to print the disposition diagram node ID alongside each
/// Taffy node.
pub struct TaffyTreeFmt;

impl TaffyTreeFmt {
    /// Writes a string representation of a Taffy tree to `buffer`.
    pub fn fmt(buffer: &mut String, taffy_node_mappings: &TaffyNodeMappings) {
        let TaffyNodeMappings {
            taffy_tree,
            node_inbuilt_to_taffy,
            node_id_to_taffy: _,
            edge_spacer_taffy_nodes: _,
            entity_highlighted_spans: _,
            taffy_id_to_node,
        } = taffy_node_mappings;
        let root_taffy_node_id = node_inbuilt_to_taffy
            .get(&NodeInbuilt::Root)
            .copied()
            .expect("Expected root taffy node to exist.");
        writeln!(buffer, "TREE").expect("Failed to write taffy tree to buffer");
        Self::fmt_node(
            buffer,
            taffy_tree,
            taffy_id_to_node,
            root_taffy_node_id,
            false,
            String::new(),
        );
    }

    /// Recursively writes each node in the tree to `buffer`.
    fn fmt_node(
        buffer: &mut String,
        taffy_tree: &TaffyTree<TaffyNodeCtx>,
        taffy_id_to_node: &Map<taffy::NodeId, NodeId>,
        taffy_node_id: taffy::NodeId,
        has_sibling: bool,
        lines_string: String,
    ) {
        let layout = &taffy_tree.get_final_layout(taffy_node_id);
        let display = taffy_id_to_node
            .get(&taffy_node_id)
            .map(|node_id| Cow::Borrowed(node_id.as_str()))
            .or_else(|| {
                taffy_tree.get_node_context(taffy_node_id).map(
                    |taffy_node_ctx| match taffy_node_ctx {
                        TaffyNodeCtx::DiagramNode(diagram_node_ctx) => {
                            Cow::Borrowed(diagram_node_ctx.entity_id.as_str())
                        }
                        TaffyNodeCtx::EdgeSpacer(edge_spacer_ctx) => {
                            let edge_id = &edge_spacer_ctx.edge_id;
                            let rank = edge_spacer_ctx.rank;
                            Cow::Owned(format!("edge_spacer_{edge_id}_{rank}"))
                        }
                    },
                )
            })
            .unwrap_or_else(|| Cow::Borrowed(taffy_tree.get_debug_label(taffy_node_id)));
        let num_children = taffy_tree.child_count(taffy_node_id);

        let fork_string = if has_sibling {
            "├── "
        } else {
            "└── "
        };
        let flex_direction = taffy_tree
            .style(taffy_node_id)
            .map(|style| Cow::Owned(format!("{:?}", style.flex_direction)))
            .unwrap_or(Cow::Borrowed("unknown"));
        writeln!(
            buffer,
            "{lines}{fork} {display} {{ flex_direction: {flex_direction}, x: {x} y: {y} w: {width} h: {height} content_w: {content_width} content_h: {content_height}, padding: l: {pl} r: {pr} t: {pt} b: {pb} }}",
            lines = lines_string,
            fork = fork_string,
            display = display,
            x = layout.location.x,
            y = layout.location.y,
            width = layout.size.width,
            height = layout.size.height,
            content_width = layout.content_size.width,
            content_height = layout.content_size.height,
            pl = layout.padding.left,
            pr = layout.padding.right,
            pt = layout.padding.top,
            pb = layout.padding.bottom,
        )
        .expect("Failed to write taffy tree to buffer");
        let bar = if has_sibling { "│   " } else { "    " };
        let new_string = lines_string + bar;

        // Recurse into children.
        taffy_tree
            .child_ids(taffy_node_id)
            .enumerate()
            .for_each(|(index, child)| {
                let has_sibling = index < num_children - 1;
                Self::fmt_node(
                    buffer,
                    taffy_tree,
                    taffy_id_to_node,
                    child,
                    has_sibling,
                    new_string.clone(),
                );
            });
    }
}
