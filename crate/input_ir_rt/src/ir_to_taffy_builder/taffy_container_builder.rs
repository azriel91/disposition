use std::collections::BTreeMap;

use disposition_ir_model::{
    layout::{FlexDirection as ModelFlexDirection, NodeLayout, NodeLayouts},
    node::{NodeId, NodeInbuilt, NodeRank},
};
use disposition_model_common::{Id, Map};
use disposition_taffy_model::{
    taffy::{
        self,
        style::{FlexDirection, LengthPercentageAuto},
        AlignContent, AlignItems, Display, FlexWrap, LengthPercentage, Rect, Size, Style,
        TaffyTree,
    },
    Dimension, TaffyNodeCtx,
};
use taffy::{prelude::TaffyZero, JustifyContent, JustifyItems};

use super::taffy_node_build_context::TaffyWrapperNodeStyles;

// === Type Aliases === //

pub(crate) type NodeRankToTaffyNodeId = BTreeMap<NodeRank, Vec<taffy::NodeId>>;

// === Free Functions === //

/// Converts a model [`FlexDirection`](ModelFlexDirection) to a
/// [`taffy::style::FlexDirection`].
fn flex_direction_to_taffy(direction: ModelFlexDirection) -> FlexDirection {
    match direction {
        ModelFlexDirection::Row => FlexDirection::Row,
        ModelFlexDirection::RowReverse => FlexDirection::RowReverse,
        ModelFlexDirection::Column => FlexDirection::Column,
        ModelFlexDirection::ColumnReverse => FlexDirection::ColumnReverse,
    }
}

/// Returns the cross-axis flex direction for `flex_direction`.
///
/// `Row` / `Column` are swapped, preserving the reversed variant. This is used
/// to stack rank containers along the axis perpendicular to the one their
/// within-rank siblings are laid out on.
fn flex_direction_invert(flex_direction: FlexDirection) -> FlexDirection {
    match flex_direction {
        FlexDirection::Row => FlexDirection::Column,
        FlexDirection::Column => FlexDirection::Row,
        FlexDirection::RowReverse => FlexDirection::ColumnReverse,
        FlexDirection::ColumnReverse => FlexDirection::RowReverse,
    }
}

// === TaffyContainerBuilder === //

/// Builds the inbuilt taffy container hierarchy (Root, ThingsAndProcesses,
/// Things, Processes, Tags containers) and their rank sub-containers, and
/// computes taffy styles from `NodeLayouts`.
pub(crate) struct TaffyContainerBuilder;

impl TaffyContainerBuilder {
    /// Adds the inbuilt container nodes to the `TaffyTree`.
    ///
    /// Builds the [`NodeInbuilt`] container hierarchy:
    ///
    /// ```text
    /// Root
    ///   TagsContainer
    ///     rank_container*  (one per rank)
    ///   ThingsAndProcessesContainer
    ///     ProcessesContainer
    ///       rank_container*
    ///     ThingsContainer
    ///       rank_container*
    /// ```
    ///
    /// # Parameters
    ///
    /// * `taffy_tree`: The `TaffyTree` to add nodes to.
    /// * `taffy_id_to_node`: Reverse map updated with the new container IDs.
    /// * `node_layouts`: Flex layout computed when mapping `InputDiagram` to
    ///   `IrDiagram`.
    /// * `dimension`: Diagram dimension used to size the root node.
    /// * `thing_rank_container_ids`: Ordered rank-container and
    ///   edge-description-container IDs for the Things container.
    /// * `process_rank_container_ids`: Same for Processes.
    /// * `tag_rank_container_ids`: Same for Tags.
    pub(crate) fn build(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        taffy_id_to_node: &mut Map<taffy::NodeId, NodeId<'static>>,
        node_layouts: &NodeLayouts<'static>,
        dimension: &Dimension,
        thing_rank_container_ids: &[taffy::NodeId],
        process_rank_container_ids: &[taffy::NodeId],
        tag_rank_container_ids: &[taffy::NodeId],
    ) -> Map<NodeInbuilt, taffy::NodeId> {
        let things_container_style = {
            let container_style = Self::taffy_container_style(
                node_layouts,
                &NodeInbuilt::ThingsContainer.id(),
                Size::auto(),
            );
            Self::container_style_invert_and_stretch(container_style)
        };
        let things_container = taffy_tree
            .new_with_children(things_container_style, thing_rank_container_ids)
            .expect("`TaffyTree::new_with_children` should be infallible.");
        let processes_container_style = {
            let container_style = Self::taffy_container_style(
                node_layouts,
                &NodeInbuilt::ProcessesContainer.id(),
                Size::auto(),
            );
            Self::container_style_invert_and_stretch(container_style)
        };
        let processes_container = taffy_tree
            .new_with_children(processes_container_style, process_rank_container_ids)
            .expect("`TaffyTree::new_with_children` should be infallible.");
        let things_and_processes_container = Self::taffy_container_node(
            taffy_tree,
            node_layouts,
            NodeInbuilt::ThingsAndProcessesContainer,
            Size::auto(),
            &[processes_container, things_container],
        );
        let tags_container_style = {
            let container_style = Self::taffy_container_style(
                node_layouts,
                &NodeInbuilt::TagsContainer.id(),
                Size::auto(),
            );
            Self::container_style_invert_and_stretch(container_style)
        };
        let tags_container = taffy_tree
            .new_with_children(tags_container_style, tag_rank_container_ids)
            .expect("`TaffyTree::new_with_children` should be infallible.");

        let root = Self::taffy_container_node(
            taffy_tree,
            node_layouts,
            NodeInbuilt::Root,
            match dimension {
                Dimension::NoLimit => Size::auto(),
                _ => Size::from_lengths(dimension.width(), dimension.height()),
            },
            &[tags_container, things_and_processes_container],
        );

        let mut node_inbuilt_to_taffy = Map::new();
        node_inbuilt_to_taffy.insert(NodeInbuilt::ThingsContainer, things_container);
        node_inbuilt_to_taffy.insert(NodeInbuilt::ProcessesContainer, processes_container);
        node_inbuilt_to_taffy.insert(
            NodeInbuilt::ThingsAndProcessesContainer,
            things_and_processes_container,
        );
        node_inbuilt_to_taffy.insert(NodeInbuilt::TagsContainer, tags_container);
        node_inbuilt_to_taffy.insert(NodeInbuilt::Root, root);

        taffy_id_to_node.insert(
            things_container,
            NodeId::from(NodeInbuilt::ThingsContainer.id()),
        );
        taffy_id_to_node.insert(
            processes_container,
            NodeId::from(NodeInbuilt::ProcessesContainer.id()),
        );
        taffy_id_to_node.insert(
            things_and_processes_container,
            NodeId::from(NodeInbuilt::ThingsAndProcessesContainer.id()),
        );
        taffy_id_to_node.insert(
            tags_container,
            NodeId::from(NodeInbuilt::TagsContainer.id()),
        );
        taffy_id_to_node.insert(root, NodeId::from(NodeInbuilt::Root.id()));

        node_inbuilt_to_taffy
    }

    /// Creates rank sub-containers for first-level nodes of a given entity
    /// type, interleaved with `edge_description_container` nodes.
    ///
    /// Each rank level gets its own flex container using
    /// `rank_container_style`. `edge_description_container` nodes from
    /// `position_to_container_ids` are inserted before or after each rank
    /// container according to their position key.
    ///
    /// The returned `Vec` contains rank containers and edge_description
    /// containers in interleaved order.
    pub(crate) fn rank_containers_for_first_level_nodes_build(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        mut rank_to_taffy_ids: NodeRankToTaffyNodeId,
        rank_container_style: Style,
        position_to_container_ids: BTreeMap<Option<NodeRank>, Vec<taffy::NodeId>>,
    ) -> Vec<taffy::NodeId> {
        Self::rank_taffy_ids_reverse_if_direction_reversed(
            &rank_container_style,
            &mut rank_to_taffy_ids,
        );

        // Creates a new taffy node for each rank to be placed in the container.
        //
        // i.e.
        //
        // ```yaml
        // container_node:
        //   child_container_0: {} # nodes with rank n
        //   child_container_1: {} # nodes with rank n + 1
        //   child_container_2: {} # nodes with rank n + 2
        // ```
        let rank_to_container: BTreeMap<NodeRank, taffy::NodeId> = rank_to_taffy_ids
            .into_iter()
            .map(|(rank, taffy_ids)| {
                let container = taffy_tree
                    .new_with_children(rank_container_style.clone(), &taffy_ids)
                    .unwrap_or_else(|e| {
                        panic!(
                            "Expected to create rank container node for \
                             top-level nodes. Error: {e}"
                        )
                    });
                (rank, container)
            })
            .collect();

        Self::rank_containers_interleave(rank_to_container, position_to_container_ids)
    }

    /// Reverses each rank's child node order when the rank container uses a
    /// reversed flex direction (`RowReverse` / `ColumnReverse`).
    ///
    /// For `RankDir::BottomToTop` / `RankDir::RightToLeft`, rank containers
    /// are laid out with a reversed flex direction. The reversed direction is
    /// retained because the rank-stacking parent inverts it (see
    /// `container_style_invert_and_stretch`) to stack ranks bottom-up /
    /// right-to-left. Without compensation, the reversed direction would also
    /// render siblings within a rank in reverse declaration order, which:
    ///
    /// * is the reverse of what a human reading the input expects, and
    /// * breaks sibling-index-based heuristics (cycle edge face selection,
    ///   spacer insertion indices) that assume a smaller sibling index means an
    ///   earlier visual position.
    ///
    /// Reversing the insertion order here cancels out the reversed flex
    /// direction along the sibling axis, so visual order matches declaration
    /// order for all rank directions. This must run **after** edge spacers
    /// are inserted into `rank_to_taffy_ids` so spacers flip together with
    /// their neighbouring nodes.
    pub(crate) fn rank_taffy_ids_reverse_if_direction_reversed(
        rank_container_style: &Style,
        rank_to_taffy_ids: &mut NodeRankToTaffyNodeId,
    ) {
        if matches!(
            rank_container_style.flex_direction,
            FlexDirection::RowReverse | FlexDirection::ColumnReverse
        ) {
            rank_to_taffy_ids
                .values_mut()
                .for_each(|taffy_ids| taffy_ids.reverse());
        }
    }

    /// Interleaves rank containers with `edge_description_container` nodes.
    ///
    /// `position_to_container_ids` maps insertion positions to the
    /// edge_description_container taffy node IDs to insert there:
    ///
    /// - `None` -- inserted before all rank containers.
    /// - `Some(rank)` -- inserted after `rank_container[rank]`.
    ///
    /// Rank containers and edge_description_containers are combined into a
    /// single ordered `Vec<taffy::NodeId>`.
    pub(crate) fn rank_containers_interleave(
        rank_to_container: BTreeMap<NodeRank, taffy::NodeId>,
        position_to_container_ids: BTreeMap<Option<NodeRank>, Vec<taffy::NodeId>>,
    ) -> Vec<taffy::NodeId> {
        let mut result = Vec::new();

        // Prepend containers positioned before all rank containers.
        if let Some(before) = position_to_container_ids.get(&None) {
            result.extend(before.iter().copied());
        }

        for (rank, rank_container) in rank_to_container {
            result.push(rank_container);
            if let Some(after) = position_to_container_ids.get(&Some(rank)) {
                result.extend(after.iter().copied());
            }
        }

        result
    }

    /// Sets the flex direction to the opposite of the container style.
    ///
    /// The flex direction inversion is because the desired flex direction is
    /// set on the rank container nodes, so when the user has requested `Row`,
    /// each rank container uses the `Row` layout, and the parent of the ranked
    /// containers should be `Column`.
    fn container_style_invert_and_stretch(container_style: Style) -> Style {
        Style {
            flex_direction: flex_direction_invert(container_style.flex_direction),
            ..container_style
        }
    }

    /// Returns the style for the container that stacks a nested container
    /// node's per-rank child containers.
    ///
    /// The per-rank child containers lay their within-rank siblings along
    /// `child_container_style.flex_direction`; this stacking container arranges
    /// the rank containers themselves along the inverted axis, so higher-ranked
    /// nodes are positioned further along the diagram's rank direction. This
    /// mirrors how the top-level inbuilt containers invert the rank container
    /// direction (see `container_style_invert_and_stretch`).
    ///
    /// Padding / border / margin are zero so the stacking container only
    /// re-orients the rank axis without shifting the nested nodes' coordinates.
    /// The gap is carried over from `child_container_style` to preserve the
    /// rank-to-rank spacing previously provided by the wrapper node.
    pub(crate) fn rank_stacking_container_style(child_container_style: &Style) -> Style {
        Style {
            display: Display::Flex,
            flex_direction: flex_direction_invert(child_container_style.flex_direction),
            flex_wrap: FlexWrap::NoWrap,
            // Rank containers must not shrink below their content size,
            // matching the per-rank `child_container_style`.
            flex_shrink: 0.0,
            align_items: Some(AlignItems::FlexStart),
            align_content: Some(AlignContent::FlexStart),
            justify_items: Some(JustifyItems::FlexStart),
            justify_content: Some(JustifyContent::FlexStart),
            gap: child_container_style.gap,
            ..Default::default()
        }
    }

    /// Adds a container node to the `TaffyTree` and returns its ID.
    ///
    /// # Parameters
    ///
    /// * `taffy_tree`: `TaffyTree` to add the node to.
    /// * `node_layouts`: Flex layout / none computed when mapping the
    ///   `InputDiagram` to the `IrDiagram`.
    /// * `node_inbuilt`: The `NodeInbuilt` struct representing the node.
    /// * `max_size`: Maximum size of the node.
    /// * `child_node_ids`: IDs of child nodes to add to the container.
    fn taffy_container_node(
        taffy_tree: &mut TaffyTree<TaffyNodeCtx>,
        node_layouts: &NodeLayouts<'_>,
        node_inbuilt: NodeInbuilt,
        max_size: Size<taffy::style::Dimension>,
        child_node_ids: &[taffy::NodeId],
    ) -> taffy::NodeId {
        let tags_container_style =
            Self::taffy_container_style(node_layouts, &node_inbuilt.id(), max_size);
        taffy_tree
            .new_with_children(tags_container_style, child_node_ids)
            .expect("`TaffyTree::new_leaf_with_context` should be infallible.")
    }

    /// Returns the `taffy::Style` for container nodes and leaf nodes.
    ///
    /// The values for each style are taken from the `NodeLayouts` map.
    ///
    /// This includes:
    ///
    /// * `inbuilt` nodes: e.g. `NodeInbuilt::ThingsAndProcessesContainer`.
    /// * `rank_container` nodes: `taffy` nodes that contain the taffy container
    ///   nodes for a given rank.
    /// * `rank` nodes: The `taffy` nodes that contain the child nodes for a
    ///   given rank.
    /// * leaf nodes: the `taffy` nodes that contain the text of a diagram node.
    pub(crate) fn taffy_container_style(
        node_layouts: &NodeLayouts<'_>,
        node_id: &Id,
        max_size: Size<taffy::style::Dimension>,
    ) -> Style {
        node_layouts
            .get(node_id)
            .map(|node_layout| match node_layout {
                NodeLayout::Flex(flex_layout) => Style {
                    display: Display::Flex,
                    max_size,
                    margin: Rect {
                        left: LengthPercentageAuto::length(flex_layout.margin_left()),
                        right: LengthPercentageAuto::length(flex_layout.margin_right()),
                        top: LengthPercentageAuto::length(flex_layout.margin_top()),
                        bottom: LengthPercentageAuto::length(flex_layout.margin_bottom()),
                    },
                    padding: Rect {
                        left: LengthPercentage::length(flex_layout.padding_left()),
                        right: LengthPercentage::length(flex_layout.padding_right()),
                        top: LengthPercentage::length(flex_layout.padding_top()),
                        bottom: LengthPercentage::length(flex_layout.padding_bottom()),
                    },
                    border: Rect::length(1.0f32),
                    // We previously used `AlignItems::Stretch` because we want coordinates to be as
                    // close to the top-left corner as possible, as well as resizing each node to be
                    // as wide as the widest node which looks more visually aesthetic.
                    //
                    // We now use `AlignItems::FlexStart` to align the content to the start of the
                    // container, which ensures that the coordinates are as close to the top-left
                    // corner as possible, as well as not inadvertently stretching nodes' height.
                    //
                    // If we use `AlignItems::Center`, the coordinates
                    // may be negative when the content width exceeds the diagram dimension, and
                    // starts outside the diagram bounds.
                    align_items: Some(AlignItems::FlexStart),
                    align_content: Some(AlignContent::Start),
                    justify_items: Some(JustifyItems::Start),
                    justify_content: Some(JustifyContent::Start),
                    gap: Size::length(flex_layout.gap()),
                    flex_direction: flex_direction_to_taffy(flex_layout.direction()),
                    flex_wrap: if flex_layout.wrap() {
                        FlexWrap::Wrap
                    } else {
                        FlexWrap::NoWrap
                    },
                    ..Default::default()
                },
                NodeLayout::Leaf(leaf_layout) => Style {
                    margin: Rect {
                        left: LengthPercentageAuto::length(leaf_layout.margin_left()),
                        right: LengthPercentageAuto::length(leaf_layout.margin_right()),
                        top: LengthPercentageAuto::length(leaf_layout.margin_top()),
                        bottom: LengthPercentageAuto::length(leaf_layout.margin_bottom()),
                    },
                    padding: Rect {
                        left: LengthPercentage::length(leaf_layout.padding_left()),
                        right: LengthPercentage::length(leaf_layout.padding_right()),
                        top: LengthPercentage::length(leaf_layout.padding_top()),
                        bottom: LengthPercentage::length(leaf_layout.padding_bottom()),
                    },
                    ..Default::default()
                },
            })
            .unwrap_or_default()
    }

    /// Returns the `taffy::Style` for a wrapper node and its text node.
    ///
    /// A wrapper node contains the text leaf and one rank-based child container
    /// per rank level. The returned styles are:
    ///
    /// * `wrapper_style` -- the outer container (flex column) that holds the
    ///   text and child containers.
    /// * `text_style` -- the leaf node that is measured for text content.
    /// * `child_container_style` -- each per-rank child container inside the
    ///   wrapper.
    pub(crate) fn taffy_wrapper_node_styles(
        node_layouts: &NodeLayouts<'_>,
        node_id: &Id,
    ) -> TaffyWrapperNodeStyles {
        node_layouts
            .get(node_id)
            .map(|node_layout| match node_layout {
                NodeLayout::Flex(flex_layout) => {
                    let wrapper_style = Style {
                        display: Display::Flex,
                        max_size: Size::auto(),
                        margin: Rect {
                            left: LengthPercentageAuto::length(flex_layout.margin_left()),
                            right: LengthPercentageAuto::length(flex_layout.margin_right()),
                            top: LengthPercentageAuto::length(flex_layout.margin_top()),
                            bottom: LengthPercentageAuto::length(flex_layout.margin_bottom()),
                        },
                        padding: Rect {
                            left: LengthPercentage::length(flex_layout.padding_left()),
                            right: LengthPercentage::length(flex_layout.padding_right()),
                            top: LengthPercentage::length(flex_layout.padding_top()),
                            bottom: LengthPercentage::length(flex_layout.padding_bottom()),
                        },
                        border: Rect::length(1.0f32),
                        align_items: Some(AlignItems::FlexStart),
                        align_content: Some(AlignContent::FlexStart),
                        justify_items: Some(JustifyItems::FlexStart),
                        justify_content: Some(JustifyContent::FlexStart),
                        // Gap between the text node and the child container node.
                        gap: Size::length(flex_layout.gap()),
                        flex_direction: FlexDirection::Column,
                        flex_wrap: FlexWrap::NoWrap,
                        ..Default::default()
                    };
                    // Leaf node doesn't need much difference from wrapper style
                    let text_style = Style {
                        padding: Rect {
                            left: LengthPercentage::length(flex_layout.padding_left()),
                            right: LengthPercentage::length(flex_layout.padding_right()),
                            top: LengthPercentage::ZERO,
                            bottom: LengthPercentage::ZERO,
                        },
                        ..Default::default()
                    };
                    let child_container_style = Style {
                        display: Display::Flex,
                        max_size: Size::auto(),
                        // Rank sub-containers must not shrink below their
                        // content size; otherwise the column wrapper parent
                        // compresses them when space is tight, causing wrapped
                        // rows to overlap with the next rank container.
                        flex_shrink: 0.0,
                        gap: Size::length(flex_layout.gap()),
                        flex_direction: flex_direction_to_taffy(flex_layout.direction()),
                        flex_wrap: if flex_layout.wrap() {
                            FlexWrap::Wrap
                        } else {
                            FlexWrap::NoWrap
                        },
                        ..Default::default()
                    };

                    TaffyWrapperNodeStyles {
                        wrapper_style,
                        text_style,
                        child_container_style,
                    }
                }
                NodeLayout::Leaf(leaf_layout) => TaffyWrapperNodeStyles::new(leaf_layout),
            })
            .unwrap_or_default()
    }
}
