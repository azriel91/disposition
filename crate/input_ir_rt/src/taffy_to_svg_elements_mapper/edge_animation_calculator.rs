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
    /// * `edge_group_cycle_distance`: Total `travel` distance of all edges in
    ///   the group plus the end-of-cycle pause distance. Used as the denominator
    ///   for this edge's keyframe percentages so every edge animates at the same
    ///   pixel speed and the cycle ends with a constant pause.
    /// * `edge_group_animation_duration_total_s`: Duration of the animation for
    ///   the edges for the entire edge group, which includes the pause at the
    ///   end of the animation.
    pub(super) fn calculate(
        edge_animation_params: EdgeAnimationParams,
        edge_path_info: &EdgePathInfo<'_, '_>,
        edge_group_cycle_distance: f64,
        edge_group_animation_duration_total_s: f64,
    ) -> EdgeAnimation {
        let EdgePathInfo {
            edge_id,
            edge: _,
            edge_type,
            path: _,
            path_length,
            preceding_travel,
            ortho_protrusion_params: _,
        } = edge_path_info;
        let path_length = *path_length;
        let preceding_travel = *preceding_travel;

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

        // Derive unique animation names from the edge ID by replacing
        // underscores with hyphens (tailwind translates underscores to spaces
        // inside arbitrary values).
        let edge_id_with_hyphens = edge_id.as_str().replace('_', "-");
        let animation_name = format!("{edge_id_with_hyphens}--stroke-dashoffset");
        let arrow_head_animation_name = format!("{edge_id_with_hyphens}--arrow-head-offset");

        // The `stroke-dashoffset` span this edge animates across: from
        // `start_offset` (visible_segments_length) to `end_offset` (-trailing_gap).
        // Sizing the keyframe window by this `travel` -- rather than the constant
        // `visible_segments_length` -- is what keeps every edge in the group
        // moving at the same pixel speed: the window width (in cycle-distance
        // units) equals the distance the dash actually travels.
        let travel = visible_segments_length + trailing_gap;

        // Keyframe percentages for this edge's slot within the cycle. The edge
        // animates from `preceding_travel` to `preceding_travel + travel`, so it
        // starts exactly when the previous edge finished, and the leftover up to
        // 100% is the constant end-of-cycle pause.
        let start_pct = preceding_travel / edge_group_cycle_distance * 100.0;
        let end_pct = (preceding_travel + travel) / edge_group_cycle_distance * 100.0;

        // The arrow head tracks the comet's leading tip, which travels the full
        // path length from the `from` node to the `to` node. The tip reaches the
        // `to` node face after exactly `path_length` of travel along the path, so
        // -- in the same cycle-distance units as the body keyframes -- the head
        // stays glued to the tip by animating its `offset-distance` from 0 to
        // `path_length` over the window `[start_pct, arrow_head_node_pct]`, then
        // is held at the node face.
        //
        // The head fades from opaque to transparent over the window from the tip
        // contacting the node face (`arrow_head_node_pct`) to the trailing end of
        // the visible segment contacting it (`arrow_head_tail_pct`, a further
        // `visible_segments_length` of travel). This makes the head fully
        // invisible by the time the body segment has passed the node, so it does
        // not linger while the next edge in the group animates.
        let arrow_head_end_offset = path_length;
        let arrow_head_start_pct = start_pct;
        let arrow_head_node_pct =
            (preceding_travel + path_length) / edge_group_cycle_distance * 100.0;
        let arrow_head_tail_pct = (preceding_travel + path_length + visible_segments_length)
            / edge_group_cycle_distance
            * 100.0;

        // stroke-dashoffset values. The edge path is drawn from the `from` node
        // to the `to` node, so to animate the dash in that same direction the
        // offset runs from `visible_segments_length` (visible segments entirely
        // before the path start, near the `from` node) to `-trailing_gap`
        // (visible segments entirely past the path end, near the `to` node).
        let start_offset = visible_segments_length;
        let end_offset = -trailing_gap;

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

        // Build the arrowhead @keyframes rule.
        //
        // The arrow head travels along the offset-path (the forward edge path,
        // `from` -> `to`) at the same pixel speed as the body, staying solid
        // (opacity: 1) until its tip contacts the `to` node face at
        // `arrow_head_node_pct`. It then fades out (opacity: 1 -> 0) while held at
        // the node, reaching fully transparent once the trailing end of the
        // visible segment also reaches the node face (`arrow_head_tail_pct`).
        let mut arrow_head_keyframe_css = format!("@keyframes {arrow_head_animation_name} {{ ");

        if arrow_head_start_pct > 0.0 {
            let _ = write!(
                arrow_head_keyframe_css,
                "0% {{ opacity: 0.0; offset-distance: 0px; }} "
            );
        }
        let _ = write!(
            arrow_head_keyframe_css,
            "{arrow_head_start_pct:.1}% {{ opacity: 1.0; offset-distance: 0px; }} "
        );
        let _ = write!(
            arrow_head_keyframe_css,
            "{arrow_head_node_pct:.1}% {{ opacity: 1.0; offset-distance: {arrow_head_end_offset:.1}px; }} "
        );
        let _ = write!(
            arrow_head_keyframe_css,
            "{arrow_head_tail_pct:.1}% {{ opacity: 0.0; offset-distance: {arrow_head_end_offset:.1}px; }} "
        );
        if arrow_head_tail_pct < 100.0 {
            let _ = write!(
                arrow_head_keyframe_css,
                "100% {{ opacity: 0.0; offset-distance: {arrow_head_end_offset:.1}px; }} "
            );
        }
        arrow_head_keyframe_css.push('}');
        arrow_head_keyframe_css.push('\n');

        EdgeAnimation {
            dasharray,
            keyframe_css,
            animation_name,
            edge_animation_duration_s: edge_group_animation_duration_total_s,
            arrow_head_keyframe_css,
            arrow_head_animation_name,
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
    /// The edge path is drawn from the `from` node to the `to` node and the
    /// dash animates in that same direction, so the segments are ordered
    /// smallest first and largest last: the largest dash leads the motion at the
    /// `to` end (the comet's head), with the smaller dashes trailing behind it.
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
            segments.to_vec()
        } else {
            segments.iter().copied().rev().collect()
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
