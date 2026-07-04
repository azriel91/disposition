use disposition_model_common::{edge::EdgeCurvature, RankDir};
use disposition_svg_model::SvgNodeInfo;
use kurbo::BezPath;

use disposition_ir_model::node::NodeFace;

use crate::taffy_to_svg_elements_mapper::{
    edge_model::EdgeType,
    edge_path_builder_pass_1::{
        EdgeFaceOffset, EdgePathBuilderPass1, NodeEdgeGeometry, SpacerCoordinates,
        BIDIRECTIONAL_OFFSET_RATIO, CURVE_CONTROL_RATIO,
    },
    ortho_protrusion_calculator::OrthoProtrusionCalculator,
};

use disposition_svg_model::OrthoProtrusionParams;

use self::{
    edge_path_builder_pass_2_curve::EdgePathBuilderPass2Curve,
    edge_path_builder_pass_2_ortho::EdgePathBuilderPass2Ortho,
};

mod edge_path_builder_pass_2_curve;
pub(super) mod edge_path_builder_pass_2_ortho;

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
/// Self-loops route through the curvature-specific builders using the
/// rank-direction face for both contacts (a U-shape for `Orthogonal`, a
/// curved loop for `Curved`). Contained-edge special cases are handled
/// directly (they ignore spacers and curvature mode).
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
    /// Contained-edge special cases always use curved paths regardless
    /// of `edge_curvature`. Self-loops follow `edge_curvature`: an
    /// orthogonal U-shape with protrusions, or a curved loop.
    ///
    /// # Parameters
    ///
    /// * `edge_curvature`: `Curved` for smooth bezier segments, `Orthogonal`
    ///   for 90-degree lines with arc corners.
    /// * `spacers`: intermediate spacer coordinates the edge must pass through,
    ///   e.g. `&[SpacerCoordinates { entry_x: 150.0, entry_y: 200.0, exit_x:
    ///   150.0, exit_y: 205.0 }]`.
    /// * `ortho_protrusion`: precomputed protrusion lengths for orthogonal edge
    ///   endpoints. Ignored when `edge_curvature` is `Curved`.
    /// * `face_override`: when `Some`, overrides the automatic face selection.
    ///   This is used to propagate the cycle-aware faces chosen in pass 1 so
    ///   that pass 2 produces a consistent path. When `None` the faces are
    ///   re-derived from the relative node positions.
    /// * `description_contact`: the edge's own description box contact (see
    ///   `SpacerCoordinatesResolver::description_contact_resolve`), applied
    ///   unconditionally regardless of `edge_curvature`. `Curved`/`Orthogonal`
    ///   already see it folded into `spacers`; this parameter exists so
    ///   `DirectStraight`/`DirectCurved` -- which otherwise ignore `spacers`
    ///   entirely -- honour it too. `None` for edges without a description.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn build(
        edge_curvature: EdgeCurvature,
        rank_dir: RankDir,
        from_info: &SvgNodeInfo,
        to_info: &SvgNodeInfo,
        edge_type: EdgeType,
        face_offset: EdgeFaceOffset,
        spacers: &[SpacerCoordinates],
        ortho_protrusion: &OrthoProtrusionParams,
        face_override: Option<(NodeFace, NodeFace)>,
        description_contact: Option<SpacerCoordinates>,
    ) -> BezPath {
        // Self-loops route through the curvature-specific builders below,
        // using the duplicated pass-1 face for both contacts. The
        // contained-edge check must be skipped for them because a node
        // geometrically contains itself.
        let is_self_loop = from_info.node_id == to_info.node_id;

        // Check if from is contained inside to (curvature mode and
        // spacers ignored).
        if !is_self_loop && EdgePathBuilderPass1::is_node_contained_in(from_info, to_info) {
            return EdgePathBuilderPass1::build_contained_edge_path(
                from_info,
                to_info,
                CURVE_CONTROL_RATIO,
            );
        }

        // Determine circle geometry for from/to nodes.
        let from_geom = EdgePathBuilderPass1::node_edge_geometry(from_info);
        let to_geom = EdgePathBuilderPass1::node_edge_geometry(to_info);

        // Determine which faces to use based on relative positions, or use
        // the pre-computed override from pass 1 (e.g. for cycle edges and
        // self-loops, where both contacts share the rank-direction face).
        let (from_face, to_face) = face_override.unwrap_or_else(|| {
            if is_self_loop {
                let face = EdgePathBuilderPass1::self_loop_face(rank_dir);
                (face, face)
            } else {
                EdgePathBuilderPass1::select_edge_faces(rank_dir, from_info, to_info)
            }
        });

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

        // Apply bidirectional offset. Skipped per-endpoint when that
        // endpoint's contact is already label-based: the label offset
        // already separates the pair's two contacts, so stacking the
        // bidirectional shift on top would push the contact past the
        // node's own face bounds (see `EdgeFaceOffset::from_offset_is_label`).
        if edge_type == EdgeType::PairRequest || edge_type == EdgeType::PairResponse {
            let offset_direction = if edge_type == EdgeType::PairResponse {
                1.0
            } else {
                -1.0
            };

            // Move start point down if this is the `PairRequest` edge.
            if !face_offset.from_offset_is_label {
                match from_face {
                    NodeFace::Right | NodeFace::Left => {
                        start_y += from_info.height_collapsed
                            * BIDIRECTIONAL_OFFSET_RATIO
                            * offset_direction;
                    }
                    NodeFace::Top | NodeFace::Bottom => {
                        start_x += from_info.width * BIDIRECTIONAL_OFFSET_RATIO * offset_direction;
                    }
                }
            }

            // Move end point down if this is the `PairResponse` edge.
            if !face_offset.to_offset_is_label {
                match to_face {
                    NodeFace::Right | NodeFace::Left => {
                        end_y += to_info.height_collapsed
                            * BIDIRECTIONAL_OFFSET_RATIO
                            * offset_direction;
                    }
                    NodeFace::Top | NodeFace::Bottom => {
                        end_x += to_info.width * BIDIRECTIONAL_OFFSET_RATIO * offset_direction;
                    }
                }
            }
        }

        // Defensive clamp: keep the contact point within the node's own
        // face span regardless of which mechanism produced the offset
        // (label offset, bidirectional pair offset, collision separation).
        EdgePathBuilderPass1::face_contact_clamp(&mut start_x, &mut start_y, from_face, from_info);
        EdgePathBuilderPass1::face_contact_clamp(&mut end_x, &mut end_y, to_face, to_info);

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

        // Direct-curvature edges get no protrusion/spacer routing at all, so
        // without help their contact point curves away from the node
        // immediately. Give each endpoint a short straight stub -- sized from
        // the node's own envelope clearance on that face (the same quantity
        // `OrthoProtrusionCalculator` uses to size protrusions for
        // spacer-routed edges) -- so the path still travels out through the
        // node's own edge-label region before curving toward the other
        // endpoint. Zero on faces with no label (clearance is `0.0` there),
        // so unlabeled direct edges are unaffected.
        let from_stub_len = OrthoProtrusionCalculator::own_envelope_clearance(from_info, from_face);
        let to_stub_len = OrthoProtrusionCalculator::own_envelope_clearance(to_info, to_face);

        // === Delegate to curvature-specific builder === //

        match edge_curvature {
            EdgeCurvature::Curved => {
                if is_self_loop {
                    EdgePathBuilderPass1::self_loop_path_build(
                        from_info,
                        from_face,
                        edge_type,
                        face_offset.from_offset,
                        face_offset.to_offset,
                    )
                } else if spacers.is_empty() {
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
                        start_x,
                        start_y,
                        end_x,
                        end_y,
                        from_face,
                        to_face,
                        ortho_protrusion,
                    )
                } else {
                    EdgePathBuilderPass2Ortho::build_spacer_edge_path(
                        start_x,
                        start_y,
                        end_x,
                        end_y,
                        from_face,
                        to_face,
                        spacers,
                        ortho_protrusion,
                    )
                }
            }
            // Direct variants draw straight from the `from` node to the `to`
            // node, ignoring `spacers` entirely -- except for
            // `description_contact`, the one waypoint applied regardless of
            // curvature (see the parameter doc above).
            EdgeCurvature::DirectStraight => {
                if is_self_loop {
                    EdgePathBuilderPass1::self_loop_path_build(
                        from_info,
                        from_face,
                        edge_type,
                        face_offset.from_offset,
                        face_offset.to_offset,
                    )
                } else if let Some(contact) = description_contact {
                    EdgePathBuilderPass1::build_straight_edge_path_via_waypoint(
                        start_x, start_y, end_x, end_y, contact,
                    )
                } else {
                    EdgePathBuilderPass1::build_straight_edge_path_with_stubs(
                        start_x,
                        start_y,
                        end_x,
                        end_y,
                        from_face,
                        to_face,
                        from_stub_len,
                        to_stub_len,
                    )
                }
            }
            EdgeCurvature::DirectCurved => {
                if is_self_loop {
                    EdgePathBuilderPass1::self_loop_path_build(
                        from_info,
                        from_face,
                        edge_type,
                        face_offset.from_offset,
                        face_offset.to_offset,
                    )
                } else if let Some(contact) = description_contact {
                    // Reuses the existing curved spacer-passthrough builder
                    // (the same one `Curved` uses above) instead of new
                    // bezier code -- a direct-curved edge with a description
                    // simply gets one waypoint.
                    EdgePathBuilderPass2Curve::build_spacer_edge_path(
                        start_x,
                        start_y,
                        end_x,
                        end_y,
                        from_face,
                        to_face,
                        &[contact],
                    )
                } else {
                    EdgePathBuilderPass1::build_curved_edge_path_with_stubs(
                        start_x,
                        start_y,
                        end_x,
                        end_y,
                        from_face,
                        to_face,
                        CURVE_CONTROL_RATIO,
                        from_stub_len,
                        to_stub_len,
                    )
                }
            }
        }
    }
}
