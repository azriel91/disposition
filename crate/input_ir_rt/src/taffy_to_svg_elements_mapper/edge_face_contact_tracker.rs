use disposition_ir_model::node::NodeId;
use disposition_model_common::Map;

use super::edge_model::NodeFace;

/// Key identifying a specific face of a specific node.
///
/// Used to group edges that connect to the same face of the same node,
/// so that their contact points can be spread out evenly.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct NodeFaceKey<'id> {
    node_id: NodeId<'id>,
    face: NodeFace,
}

/// Tracks which edges contact which face of which node, and computes
/// pixel offsets so that multiple edges sharing a face are spread evenly
/// around the face's midpoint.
///
/// # Algorithm
///
/// 1. Register every edge contact point (node + face) via `contact_register`.
/// 2. After all contacts are registered, call `offset_calculate` for each
///    contact in sorted order to obtain the pixel offset for that contact
///    point.
///
/// The offsets are distributed symmetrically around the midpoint of the
/// face. For `n` contact points the offsets are:
///
/// ```text
/// offset_i = (i - (n - 1) / 2.0) * gap
/// ```
///
/// where `i` is the 0-based index assigned in sorted order.
///
/// ## Gap calculation
///
/// * Start with 10% of the face length (width for Top/Bottom, height for
///   Left/Right).
/// * Clamp to a minimum of 5 px.
/// * If `n * gap > face_length`, shrink to `face_length / n` so that all
///   contact points fit within the face.
#[derive(Clone, Debug)]
pub(super) struct EdgeFaceContactTracker<'id> {
    /// For each (node, face), the total number of contact points.
    contact_counts: Map<NodeFaceKey<'id>, usize>,
    /// For each (node, face), the next index to hand out (auto-incrementing
    /// counter used by `offset_calculate`).
    contact_next_index: Map<NodeFaceKey<'id>, usize>,
}

/// Minimum gap in pixels between adjacent edge contact points on the
/// same node face.
const CONTACT_GAP_MIN_PX: f32 = 5.0;

/// Gap as a fraction of the face length (10%).
const CONTACT_GAP_RATIO: f32 = 0.10;

impl<'id> EdgeFaceContactTracker<'id> {
    /// Creates an empty tracker.
    pub(super) fn new() -> Self {
        Self {
            contact_counts: Map::new(),
            contact_next_index: Map::new(),
        }
    }

    /// Registers one contact point on the given `face` of `node_id`.
    ///
    /// Call this once per edge endpoint. For a self-loop edge that
    /// touches the same face twice, call this twice.
    pub(super) fn contact_register(&mut self, node_id: NodeId<'id>, face: NodeFace) {
        let key = NodeFaceKey { node_id, face };
        *self.contact_counts.entry(key).or_insert(0) += 1;
    }

    /// Calculates the pixel offset for the next contact point on the
    /// given face of `node_id`.
    ///
    /// Each successive call for the same (node, face) pair returns the
    /// offset for the next index. Contacts must be presented in the
    /// desired sorted order.
    ///
    /// `face_length` is the length of the face in pixels -- width for
    /// `NodeFace::Top` / `NodeFace::Bottom`, height for
    /// `NodeFace::Left` / `NodeFace::Right`.
    ///
    /// Returns the signed pixel offset from the face midpoint.
    /// Negative values go left/up, positive values go right/down.
    pub(super) fn offset_calculate(
        &mut self,
        node_id: &NodeId<'id>,
        face: NodeFace,
        face_length: f32,
    ) -> f32 {
        let key = NodeFaceKey {
            node_id: node_id.clone(),
            face,
        };

        let count = self.contact_counts.get(&key).copied().unwrap_or(1);
        let index = {
            let idx = self.contact_next_index.entry(key).or_insert(0);
            let current = *idx;
            *idx += 1;
            current
        };

        Self::offset_for_index(index, count, face_length)
    }

    /// Computes the pixel offset for the `index`-th contact out of
    /// `count` total contacts on a face of the given `face_length`.
    ///
    /// This is a pure function suitable for testing.
    fn offset_for_index(index: usize, count: usize, face_length: f32) -> f32 {
        if count <= 1 {
            return 0.0;
        }

        let gap = Self::gap_calculate(count, face_length);
        let center = (count as f32 - 1.0) / 2.0;
        (index as f32 - center) * gap
    }

    /// Computes the gap between adjacent contact points.
    ///
    /// * 10% of `face_length`, clamped to at least 5 px.
    /// * Shrunk if all contacts would exceed the face length.
    fn gap_calculate(count: usize, face_length: f32) -> f32 {
        let gap = (face_length * CONTACT_GAP_RATIO).max(CONTACT_GAP_MIN_PX);

        if count as f32 * gap > face_length {
            face_length / count as f32
        } else {
            gap
        }
    }

    /// Resets the per-face index counters so that a second pass can
    /// re-iterate contacts in a (possibly different) order.
    ///
    /// Contact counts are preserved.
    pub(super) fn indices_reset(&mut self) {
        self.contact_next_index.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_contact_offset_is_zero() {
        let offset = EdgeFaceContactTracker::offset_for_index(0, 1, 100.0);
        assert!((offset).abs() < f32::EPSILON);
    }

    #[test]
    fn two_contacts_are_symmetric() {
        let o0 = EdgeFaceContactTracker::offset_for_index(0, 2, 100.0);
        let o1 = EdgeFaceContactTracker::offset_for_index(1, 2, 100.0);
        assert!((o0 + o1).abs() < f32::EPSILON, "offsets should sum to 0");
        assert!(o0 < 0.0, "first offset should be negative");
        assert!(o1 > 0.0, "second offset should be positive");
    }

    #[test]
    fn three_contacts_middle_is_zero() {
        let o0 = EdgeFaceContactTracker::offset_for_index(0, 3, 100.0);
        let o1 = EdgeFaceContactTracker::offset_for_index(1, 3, 100.0);
        let o2 = EdgeFaceContactTracker::offset_for_index(2, 3, 100.0);
        assert!(o0 < 0.0);
        assert!((o1).abs() < f32::EPSILON, "middle offset should be 0");
        assert!(o2 > 0.0);
    }

    #[test]
    fn gap_clamped_to_minimum() {
        // face_length = 20, 10% = 2.0, should clamp to 5.0
        let gap = EdgeFaceContactTracker::gap_calculate(2, 20.0);
        assert!((gap - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn gap_shrinks_when_contacts_exceed_face() {
        // face_length = 30, 10% = 3 -> clamped to 5, but 10 * 5 = 50 > 30
        // so gap = 30 / 10 = 3.0
        let gap = EdgeFaceContactTracker::gap_calculate(10, 30.0);
        assert!((gap - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn gap_uses_ratio_when_sufficient() {
        // face_length = 200, 10% = 20, 3 * 20 = 60 <= 200 -> gap = 20
        let gap = EdgeFaceContactTracker::gap_calculate(3, 200.0);
        assert!((gap - 20.0).abs() < f32::EPSILON);
    }
}
