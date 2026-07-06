use std::cmp::Ordering;

use disposition_ir_model::{edge::EdgeId, node::NodeRank};

/// Context data stored with edge spacer nodes in the taffy tree.
///
/// Edge spacer nodes are inserted at intermediate rank levels for
/// edges that cross multiple ranks. They participate in the flex
/// layout so that the edge path can be routed through their
/// positions, reducing the chance of edges being drawn over other
/// nodes.
#[derive(Clone, Debug, PartialEq)]
pub struct EdgeSpacerCtx {
    /// The edge ID this spacer is for.
    pub edge_id: EdgeId<'static>,
    /// Rank that the spacer node is at.
    pub rank: NodeRank,
    /// For a same-rank (cycle edge) crossing spacer only: this edge's own
    /// `sibling_index_from.cmp(&sibling_index_to)` at the LCA depth.
    ///
    /// `None` for every other spacer kind. `Some(_)` tells
    /// `SpacerCoordinatesResolver::resolve` to resolve this spacer's
    /// coordinates via
    /// `EdgeSpacerCoordinatesCalculator::calculate_description_thread_same_rank`
    /// (which threads through the leaf's own rect, swapping entry/exit to
    /// match *this* edge's travel direction) instead of the direction-oblivious
    /// generic `calculate`, so the spacer's own vertical (or horizontal)
    /// passthrough starts from whichever of its two ends this edge actually
    /// approaches first -- see
    /// `EdgeSpacerBuilder::build_edge_desc_container_spacers_for_edge_same_rank`.
    pub same_rank_sibling_index_from_cmp_to: Option<Ordering>,
}
