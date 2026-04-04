use disposition_model_common::{edge::EdgeCurvature, RankDir};
use disposition_svg_model::SvgNodeInfo;
use kurbo::BezPath;

use crate::taffy_to_svg_elements_mapper::{
    edge_model::{EdgeType, NodeFace},
    edge_path_builder::{
        EdgeFaceOffset, EdgePathBuilderPass1, NodeEdgeGeometry, SpacerCoordinates,
        BIDIRECTIONAL_OFFSET_RATIO, CURVE_CONTROL_RATIO, SELF_LOOP_X_EXTENSION_RATIO,
        SELF_LOOP_X_OFFSET_RATIO, SELF_LOOP_Y_EXTENSION_RATIO,
    },
};

use self::{
    edge_path_builder_pass2_curve::EdgePathBuilderPass2Curve,
    edge_path_builder_pass2_ortho::EdgePathBuilderPass2Ortho,
};

mod edge_path_builder_pass2_curve;
mod edge_path_builder_pass2_ortho;

/// Direction specification for a curve or orthogonal endpoint: either
/// an outward node face normal or an explicit unit direction vector.
///
/// Used by the curve and orthogonal segment builders to compute control
/// points or corner positions.
///
/// # Examples
///
/// ```text
/// FaceOrDirection::Face(NodeFace::Bottom)
/// FaceOrDirection::Direction((0.0, -1.0))
/// ```
#[derive(Clone, Copy, Debug)]
pub(in crate::taffy_to_svg_elements_mapper) enum FaceOrDirection {
    /// Outward normal of a node face, e.g. `NodeFace::Bottom`.
    Face(NodeFace),
    /// Explicit unit direction vector, e.g. `(0.0, 1.0)` for downward.
    Direction((f32, f32)),
}

/// Builds pass-2 edge paths with per-face offsets and intermediate
/// spacer passthrough segments.
///
/// Delegates the segment-drawing strategy to
/// `EdgePathBuilderPass2Curve` or `EdgePathBuilderPass2Ortho` based on
/// the provided `EdgeCurvature`.
///
/// Self-loops and contained-edge special cases are handled directly
/// (they ignore spacers and curvature mode).
#[derive(Clone, Copy, Debug)]
pub(super) struct EdgePathBuilderPass2;

impl EdgePathBuilderPass2 {
    /// Builds the SVG path with per-face contact point offsets and
    /// intermediate spacer passthrough segments, using the given
    /// `edge_curvature` to select the drawing strategy.
    ///
    /// When `spacers` is empty the path connects the two nodes
    /// directly (curved or orthogonal depending on `edge_curvature`).
    /// Otherwise the path routes through spacers with curved or
    /// orthogonal inter-spacer segments.
    ///
    /// Self-loops and contained-edge special cases always use curved
    /// paths regardless of `edge_curvature`.
    ///
    /// # Parameters
    ///
    /// * `edge_curvature` -- `Curved` for smooth bezier segments, `Orthogonal`
    ///   for 90-degree lines with arc corners.
    /// * `spacers` -- intermediate spacer coordinates the edge must pass
    ///   through, e.g. `&[SpacerCoordinates { entry_x: 150.0, entry_y: 200.0,
    ///   exit_x: 150.0, exit_y: 205.0 }]`.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn build(
        edge_curvature: EdgeCurvature,
        rank_dir: RankDir,
        from_info: &SvgNodeInfo,
        to_info: &SvgNodeInfo,
        edge_type: EdgeType,
        face_offset: EdgeFaceOffset,
        spacers: &[SpacerCoordinates],
    ) -> BezPath {
        // Handle self-loop case (curvature mode and spacers ignored).
        if from_info.node_id == to_info.node_id {
            return EdgePathBuilderPass1::build_self_loop_path(
                from_info,
                edge_type,
                SELF_LOOP_X_OFFSET_RATIO,
                SELF_LOOP_Y_EXTENSION_RATIO,
                SELF_LOOP_X_EXTENSION_RATIO,
            );
        }

        // Check if from is contained inside to (curvature mode and
        // spacers ignored).
        if EdgePathBuilderPass1::is_node_contained_in(from_info, to_info) {
            return EdgePathBuilderPass1::build_contained_edge_path(
                from_info,
                to_info,
                CURVE_CONTROL_RATIO,
            );
        }

        // Determine circle geometry for from/to nodes.
        let from_geom = EdgePathBuilderPass1::node_edge_geometry(from_info);
        let to_geom = EdgePathBuilderPass1::node_edge_geometry(to_info);

        // Determine which faces to use based on relative positions.
        let (from_face, to_face) =
            EdgePathBuilderPass1::select_edge_faces(rank_dir, from_info, to_info);

        // Get base connection points.
        let (mut start_x, mut start_y) =
            EdgePathBuilderPass1::get_face_center(from_info, from_face);
        let (mut end_x, mut end_y) = EdgePathBuilderPass1::get_face_center(to_info, to_face);

        // Apply face contact offsets (spread edges along the face).
        EdgePathBuilderPass1::face_offset_apply(
            &mut start_x,
            &mut start_y,
            from_face,
            face_offset.from_offset,
        );
        EdgePathBuilderPass1::face_offset_apply(
            &mut end_x,
            &mut end_y,
            to_face,
            face_offset.to_offset,
        );

        // Apply bidirectional offset.
        if edge_type == EdgeType::PairRequest || edge_type == EdgeType::PairResponse {
            let offset_direction = if edge_type == EdgeType::PairResponse {
                1.0
            } else {
                -1.0
            };

            // Move start point down if this is the `PairRequest` edge.
            match from_face {
                NodeFace::Right | NodeFace::Left => {
                    start_y +=
                        from_info.height_collapsed * BIDIRECTIONAL_OFFSET_RATIO * offset_direction;
                }
                NodeFace::Top | NodeFace::Bottom => {
                    start_x += from_info.width * BIDIRECTIONAL_OFFSET_RATIO * offset_direction;
                }
            }

            // Move end point down if this is the `PairResponse` edge.
            match to_face {
                NodeFace::Right | NodeFace::Left => {
                    end_y +=
                        to_info.height_collapsed * BIDIRECTIONAL_OFFSET_RATIO * offset_direction;
                }
                NodeFace::Top | NodeFace::Bottom => {
                    end_x += to_info.width * BIDIRECTIONAL_OFFSET_RATIO * offset_direction;
                }
            }
        }

        // If either node has a circle, snap the connection point to the
        // circle perimeter instead of the rectangular face center.
        if let NodeEdgeGeometry::Circle { cx, cy, radius } = from_geom {
            let (sx, sy) =
                EdgePathBuilderPass1::circle_perimeter_point(cx, cy, radius, end_x, end_y);
            start_x = sx;
            start_y = sy;
        }
        if let NodeEdgeGeometry::Circle { cx, cy, radius } = to_geom {
            let (ex, ey) =
                EdgePathBuilderPass1::circle_perimeter_point(cx, cy, radius, start_x, start_y);
            end_x = ex;
            end_y = ey;
        }

        // === Delegate to curvature-specific builder === //

        match edge_curvature {
            EdgeCurvature::Curved => {
                if spacers.is_empty() {
                    EdgePathBuilderPass1::build_curved_edge_path(
                        start_x,
                        start_y,
                        end_x,
                        end_y,
                        from_face,
                        to_face,
                        CURVE_CONTROL_RATIO,
                    )
                } else {
                    EdgePathBuilderPass2Curve::build_spacer_edge_path(
                        start_x, start_y, end_x, end_y, from_face, to_face, spacers,
                    )
                }
            }
            EdgeCurvature::Orthogonal => {
                if spacers.is_empty() {
                    EdgePathBuilderPass2Ortho::build_ortho_edge_path(
                        start_x, start_y, end_x, end_y, from_face, to_face,
                    )
                } else {
                    EdgePathBuilderPass2Ortho::build_spacer_edge_path(
                        start_x, start_y, end_x, end_y, from_face, to_face, spacers,
                    )
                }
            }
        }
    }
}
