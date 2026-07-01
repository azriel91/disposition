/// Computes pixel offsets that spread multiple edges sharing a single node
/// face symmetrically around the face's midpoint.
///
/// Callers collect the contacts on each `(node, face)` in sorted order, then
/// call [`offset_for_index`](Self::offset_for_index) per contact to obtain its
/// signed pixel offset. The offsets are distributed symmetrically around the
/// midpoint of the face. For `n` contact points the offsets are:
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
/// * Clamp to a minimum of `CONTACT_GAP_MIN_PX`.
/// * If `n * gap > face_length`, shrink to `face_length / n` so that all
///   contact points fit within the face.
pub(super) struct EdgeFaceContactTracker;

/// Minimum gap in pixels between adjacent edge contact points on the
/// same node face.
///
/// May need to be twice as big as `ArrowHeadBuilder::ARROW_HEAD_HALF_WIDTH`.
pub(super) const CONTACT_GAP_MIN_PX: f32 = 12.0;

/// Gap as a fraction of the face length (10%).
const CONTACT_GAP_RATIO: f32 = 0.10;

impl EdgeFaceContactTracker {
    /// Computes the pixel offset for the `index`-th contact out of
    /// `count` total contacts on a face of the given `face_length`.
    ///
    /// `face_length` is the length of the face in pixels -- width for
    /// `NodeFace::Top` / `NodeFace::Bottom`, height for
    /// `NodeFace::Left` / `NodeFace::Right`.
    ///
    /// Returns the signed pixel offset from the face midpoint. Negative
    /// values go left/up, positive values go right/down.
    pub(super) fn offset_for_index(
        contact_index: usize,
        contact_count: usize,
        face_length: f32,
    ) -> f32 {
        if contact_count <= 1 {
            return 0.0;
        }

        let gap = Self::gap_calculate(contact_count, face_length);
        let center = (contact_count as f32 - 1.0) / 2.0;
        (contact_index as f32 - center) * gap
    }

    /// Computes the gap between adjacent contact points.
    ///
    /// * 10% of `face_length`, clamped to at least `CONTACT_GAP_MIN_PX`.
    /// * Shrunk if all contacts would exceed the face length.
    pub(super) fn gap_calculate(contact_count: usize, face_length: f32) -> f32 {
        let gap = (face_length * CONTACT_GAP_RATIO).max(CONTACT_GAP_MIN_PX);

        if contact_count as f32 * gap > face_length {
            face_length / contact_count as f32
        } else {
            gap
        }
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
        let offset_0 = EdgeFaceContactTracker::offset_for_index(0, 2, 100.0);
        let offset_1 = EdgeFaceContactTracker::offset_for_index(1, 2, 100.0);
        assert!(
            (offset_0 + offset_1).abs() < f32::EPSILON,
            "offsets should sum to 0"
        );
        assert!(offset_0 < 0.0, "first offset should be negative");
        assert!(offset_1 > 0.0, "second offset should be positive");
    }

    #[test]
    fn three_contacts_middle_is_zero() {
        let offset_0 = EdgeFaceContactTracker::offset_for_index(0, 3, 100.0);
        let offset_1 = EdgeFaceContactTracker::offset_for_index(1, 3, 100.0);
        let offset_2 = EdgeFaceContactTracker::offset_for_index(2, 3, 100.0);
        assert!(offset_0 < 0.0);
        assert!((offset_1).abs() < f32::EPSILON, "middle offset should be 0");
        assert!(offset_2 > 0.0);
    }

    #[test]
    fn gap_clamped_to_minimum() {
        // face_length = 40, 10% = 4.0, should clamp to CONTACT_GAP_MIN_PX
        // (12.0); 2 * 12 = 24 <= 40 so no shrink applies.
        let gap = EdgeFaceContactTracker::gap_calculate(2, 40.0);
        assert!((gap - CONTACT_GAP_MIN_PX).abs() < f32::EPSILON);
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
