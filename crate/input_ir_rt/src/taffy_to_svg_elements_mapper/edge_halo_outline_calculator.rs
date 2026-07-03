use kurbo::{offset::offset_cubic, BezPath, CubicBez, PathEl, PathSeg};

/// Half-width offset applied to each rail from the edge's centerline path,
/// in pixels.
///
/// This is deliberately decoupled from the halo's theme-resolved
/// `StrokeWidth` (default `"8"`), mirroring the existing precedent of
/// `EdgePathLocusCalculator::LOCUS_STROKE_WIDTH` -- threading the
/// theme-resolved value through would require passing resolved theme styles
/// into this geometry-building pass, which does not have that dependency
/// today.
const HALO_OUTLINE_OFFSET_PX: f64 = 4.0;

/// Accuracy tolerance for offset-curve approximation.
const HALO_OUTLINE_TOLERANCE: f64 = 0.1;

/// Two open "rail" paths running along the long sides of an interaction
/// edge's halo ribbon.
///
/// Each rail is an open `BezPath` -- no cap is drawn at either end -- so the
/// halo's short ends are left unlined.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct EdgeHaloOutlineRails {
    /// Rail offset to one side of the edge's centerline (`+half_width`).
    pub(super) rail_a: BezPath,
    /// Rail offset to the other side of the edge's centerline (`-half_width`).
    pub(super) rail_b: BezPath,
}

/// Computes the halo outline rails for an interaction edge's path.
///
/// The rails are the parallel offset curves running along the long sides of
/// the halo ribbon, at `HALO_OUTLINE_OFFSET_PX` on either side of the edge's
/// centerline, with no cap drawn at the path's start or end.
#[derive(Clone, Copy, Debug)]
pub(super) struct EdgeHaloOutlineCalculator;

impl EdgeHaloOutlineCalculator {
    /// Computes the two open rail `BezPath`s offset from `edge_path`'s
    /// centerline.
    pub(super) fn calculate(edge_path: &BezPath) -> EdgeHaloOutlineRails {
        let rail_a = Self::calculate_rail(edge_path, HALO_OUTLINE_OFFSET_PX);
        let rail_b = Self::calculate_rail(edge_path, -HALO_OUTLINE_OFFSET_PX);

        EdgeHaloOutlineRails { rail_a, rail_b }
    }

    /// Builds one open rail, offset by signed distance `d` from
    /// `edge_path`'s centerline.
    ///
    /// Each segment of `edge_path` is offset independently via
    /// [`offset_cubic`] into a reused scratch buffer -- `offset_cubic`
    /// truncates and rewrites its `result` path on every call, so it cannot
    /// accumulate multiple segments itself. Consecutive segments' offsets
    /// are stitched together with a straight bevel join (`line_to` from the
    /// previous segment's offset end to the next segment's offset start); no
    /// cap is added before the first segment or after the last, leaving the
    /// rail open at both ends.
    fn calculate_rail(edge_path: &BezPath, d: f64) -> BezPath {
        let mut rail = BezPath::new();
        let mut scratch = BezPath::new();

        edge_path.segments().for_each(|seg| {
            let cubic = Self::calculate_rail_segment_as_cubic(seg);
            offset_cubic(cubic, d, HALO_OUTLINE_TOLERANCE, &mut scratch);

            let mut scratch_elements = scratch.elements().iter().copied();
            let Some(PathEl::MoveTo(offset_start)) = scratch_elements.next() else {
                return;
            };

            if rail.elements().is_empty() {
                rail.move_to(offset_start);
            } else {
                rail.line_to(offset_start);
            }

            scratch_elements.for_each(|el| {
                if let PathEl::CurveTo(p1, p2, p3) = el {
                    rail.curve_to(p1, p2, p3);
                }
            });
        });

        rail
    }

    /// Elevates any `PathSeg` variant into a `CubicBez`, so [`offset_cubic`]
    /// can handle `Line`, `Quad`, and `Cubic` segments uniformly.
    fn calculate_rail_segment_as_cubic(seg: PathSeg) -> CubicBez {
        match seg {
            PathSeg::Line(line) => CubicBez::new(
                line.p0,
                line.p0.lerp(line.p1, 1.0 / 3.0),
                line.p0.lerp(line.p1, 2.0 / 3.0),
                line.p1,
            ),
            PathSeg::Quad(quad) => quad.raise(),
            PathSeg::Cubic(cubic) => cubic,
        }
    }
}
