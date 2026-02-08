use std::fmt::Write;

use super::edge_model::{EdgeAnimation, EdgeAnimationParams, EdgePathInfo, EdgeType};

/// Calculates stroke-dasharray animations and CSS keyframes for interaction
/// edges.
#[derive(Clone, Copy, Debug)]
pub(super) struct EdgeAnimationCalculator;

impl EdgeAnimationCalculator {
    /// Calculates the stroke-dasharray, CSS keyframes, and animation name for
    /// an interaction edge.
    ///
    /// # Parameters
    ///
    /// * `edge_animation_params`: Parameters for edge `stroke-dasharray`
    ///   animation generation.
    /// * `edge_path_info`: Path information about this edge, used to compute
    ///   timing and offset values.
    /// * `edge_group_path_or_visible_segments_length_max`: Combined length of
    ///   the paths or visible segments in the edge group (whichever is bigger).
    /// * `edge_group_animation_duration_total_s`: Duration of the animation for
    ///   the edges for the entire edge group, which includes the pause at the
    ///   end of the animation.
    pub(super) fn calculate(
        edge_animation_params: EdgeAnimationParams,
        edge_path_info: &EdgePathInfo<'_, '_>,
        edge_group_path_or_visible_segments_length_max: f64,
        edge_group_animation_duration_total_s: f64,
    ) -> EdgeAnimation {
        let EdgePathInfo {
            edge_id,
            edge: _,
            edge_type,
            path: _,
            path_length,
            preceding_visible_segments_lengths,
        } = edge_path_info;
        let path_length = *path_length;

        let is_reverse = *edge_type == EdgeType::PairResponse;

        // Generate the decreasing visible segment lengths using a geometric
        // series.
        let segments = Self::compute_dasharray_segments(edge_animation_params, is_reverse);
        let visible_segments_length = edge_animation_params.visible_segments_length;

        // Use the path length so the trailing gap fully hides the
        // edge during the invisible phase of the animation.
        let trailing_gap = path_length.max(visible_segments_length);

        // Build the dasharray string with segments in the correct order.
        let dasharray = Self::build_dasharray_string(
            &segments,
            edge_animation_params.gap_width,
            trailing_gap,
            is_reverse,
        );

        // Derive a unique animation name from the edge ID by replacing
        // underscores with hyphens (tailwind translates underscores to spaces
        // inside arbitrary values).
        let animation_name = format!("{}--stroke-dashoffset", edge_id.as_str().replace('_', "-"));

        // Keyframe percentages for this edge's slot within the cycle.
        let start_pct = preceding_visible_segments_lengths
            / edge_group_path_or_visible_segments_length_max
            * 100.0;
        let end_pct = (preceding_visible_segments_lengths + visible_segments_length)
            / edge_group_path_or_visible_segments_length_max
            * 100.0;

        // stroke-dashoffset values:
        // - start_offset: shifts visible segments entirely before the path
        // - end_offset:   shifts visible segments entirely past the path
        let start_offset = -trailing_gap;
        let end_offset = visible_segments_length;

        // Build the CSS @keyframes rule, omitting duplicate percentage entries
        // at 0% and 100% when they coincide with start_pct / end_pct.
        let mut keyframe_css = format!("@keyframes {} {{ ", animation_name);

        if start_pct > 0.0 {
            let _ = write!(
                keyframe_css,
                "0% {{ stroke-dashoffset: {start_offset:.1}; }} "
            );
        }
        let _ = write!(
            keyframe_css,
            "{start_pct:.1}% {{ stroke-dashoffset: {start_offset:.1}; }} "
        );
        let _ = write!(
            keyframe_css,
            "{end_pct:.1}% {{ stroke-dashoffset: {end_offset:.1}; }} "
        );
        if end_pct < 100.0 {
            let _ = write!(
                keyframe_css,
                "100% {{ stroke-dashoffset: {end_offset:.1}; }} "
            );
        }
        keyframe_css.push('}');
        keyframe_css.push('\n');

        EdgeAnimation {
            dasharray,
            keyframe_css,
            animation_name,
            edge_animation_duration_s: edge_group_animation_duration_total_s,
        }
    }

    /// Generates the visible segment lengths using a geometric series.
    ///
    /// Given `n` segments with ratio `r`, the first segment length `a` is
    /// computed so that:
    ///
    /// ```text
    /// a + a*r + a*r^2 + ... + a*r^(n-1) + (n-1)*gap = visible_segments_length
    /// ```
    ///
    /// Each successive segment is `r` times the previous, producing a visually
    /// decreasing pattern (e.g. long dash, medium dash, short dash, ...).
    fn compute_dasharray_segments(
        edge_animation_params: EdgeAnimationParams,
        is_reverse: bool,
    ) -> Vec<f64> {
        let n = edge_animation_params.segment_count;
        let r = edge_animation_params.segment_ratio;
        let gap = edge_animation_params.gap_width;
        let visible_segments_length = edge_animation_params.visible_segments_length;

        // Space available for visible segments after subtracting inter-segment gaps.
        let available = visible_segments_length - (n as f64 - 1.0) * gap;
        assert!(
            available > 0.0,
            "visible_segments_length ({visible_segments_length}) must be larger than the total gap \
             space ({} * {gap} = {})",
            n - 1,
            (n as f64 - 1.0) * gap,
        );

        // Sum of geometric series: a * (1 - r^n) / (1 - r)
        let weight_sum = (1.0 - r.powi(n as i32)) / (1.0 - r);
        let first = available / weight_sum;

        match is_reverse {
            false => (0..n)
                .map(|i| (first * r.powi(i as i32)).max(0.5))
                .collect(),
            true => (0..n)
                .rev()
                .map(|i| (first * r.powi(i as i32)).max(0.5))
                .collect(),
        }
    }

    /// Builds the stroke-dasharray value string from visible segments.
    ///
    /// For forward edges the segments are in decreasing order (largest first).
    /// For reverse edges the segments are in increasing order (smallest first),
    /// producing a visual "building up" effect for the response direction.
    ///
    /// The trailing gap is appended at the end so the edge is hidden during the
    /// invisible portion of the animation cycle.
    fn build_dasharray_string(
        segments: &[f64],
        gap_width: f64,
        trailing_gap: f64,
        is_reverse: bool,
    ) -> String {
        let ordered: Vec<f64> = if is_reverse {
            segments.iter().copied().rev().collect()
        } else {
            segments.to_vec()
        };

        let mut parts = Vec::with_capacity(ordered.len() * 2 + 1);
        for (i, seg) in ordered.iter().enumerate() {
            if i > 0 {
                parts.push(format!("{gap_width:.1}"));
            }
            parts.push(format!("{seg:.1}"));
        }
        parts.push(format!("{trailing_gap:.1}"));

        parts.join(",")
    }

    /// Formats a duration in seconds for use in CSS, removing unnecessary
    /// trailing zeros.
    pub(super) fn format_duration(secs: f64) -> String {
        if secs.fract() == 0.0 {
            format!("{}", secs as u64)
        } else {
            format!("{:.1}", secs)
        }
    }
}
