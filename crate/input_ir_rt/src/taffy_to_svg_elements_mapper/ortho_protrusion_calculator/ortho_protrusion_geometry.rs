use disposition_ir_model::node::NodeFace;
use disposition_svg_model::SvgNodeInfo;

/// Pure coordinate geometry helpers for orthogonal protrusion routing.
///
/// These functions translate between a node's box coordinates and the rank /
/// cross axes implied by the [`NodeFace`] an edge attaches to. They are
/// stateless and depend only on their arguments.
pub(super) struct OrthoProtrusionGeometry;

impl OrthoProtrusionGeometry {
    /// Returns the cross-axis coordinate of a node for a given face.
    ///
    /// For `Top` / `Bottom` faces the cross-axis is horizontal (X).
    /// For `Left` / `Right` faces the cross-axis is vertical (Y).
    pub(super) fn cross_axis_coord(node_x: f32, node_y: f32, face: NodeFace) -> f32 {
        match face {
            NodeFace::Top | NodeFace::Bottom => node_x,
            NodeFace::Left | NodeFace::Right => node_y,
        }
    }

    /// Returns the main-axis (rank-axis) coordinate of a point for a given
    /// face.
    ///
    /// This is the coordinate along the rank flow direction: Y for `Top` /
    /// `Bottom` faces (vertical flow), X for `Left` / `Right` faces
    /// (horizontal flow). It is the complement of [`Self::cross_axis_coord`].
    pub(super) fn main_axis_coord(x: f32, y: f32, face: NodeFace) -> f32 {
        match face {
            NodeFace::Top | NodeFace::Bottom => y,
            NodeFace::Left | NodeFace::Right => x,
        }
    }

    /// Returns the face center coordinates for a node.
    pub(super) fn face_center(info: &SvgNodeInfo<'_>, face: NodeFace) -> (f32, f32) {
        match face {
            NodeFace::Top => (info.x + info.width / 2.0, info.y),
            NodeFace::Bottom => (info.x + info.width / 2.0, info.y + info.height_collapsed),
            NodeFace::Left => (info.x, info.y + info.height_collapsed / 2.0),
            NodeFace::Right => (info.x + info.width, info.y + info.height_collapsed / 2.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cross_axis_coord_uses_x_for_horizontal_faces() {
        assert_eq!(
            3.0,
            OrthoProtrusionGeometry::cross_axis_coord(3.0, 7.0, NodeFace::Top)
        );
        assert_eq!(
            3.0,
            OrthoProtrusionGeometry::cross_axis_coord(3.0, 7.0, NodeFace::Bottom)
        );
    }

    #[test]
    fn cross_axis_coord_uses_y_for_vertical_faces() {
        assert_eq!(
            7.0,
            OrthoProtrusionGeometry::cross_axis_coord(3.0, 7.0, NodeFace::Left)
        );
        assert_eq!(
            7.0,
            OrthoProtrusionGeometry::cross_axis_coord(3.0, 7.0, NodeFace::Right)
        );
    }

    #[test]
    fn main_axis_coord_uses_y_for_horizontal_faces() {
        assert_eq!(
            7.0,
            OrthoProtrusionGeometry::main_axis_coord(3.0, 7.0, NodeFace::Top)
        );
        assert_eq!(
            7.0,
            OrthoProtrusionGeometry::main_axis_coord(3.0, 7.0, NodeFace::Bottom)
        );
    }

    #[test]
    fn main_axis_coord_uses_x_for_vertical_faces() {
        assert_eq!(
            3.0,
            OrthoProtrusionGeometry::main_axis_coord(3.0, 7.0, NodeFace::Left)
        );
        assert_eq!(
            3.0,
            OrthoProtrusionGeometry::main_axis_coord(3.0, 7.0, NodeFace::Right)
        );
    }
}
