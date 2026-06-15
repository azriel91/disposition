use disposition_input_model::DiagramFocus;

use crate::DiagramGenerated;

/// A generated diagram paired with the focus state it represents.
///
/// This is produced by `DiagramGenerator::generate_per_process_step_or_tag` (in
/// `disposition_input_ir_rt`), which generates one diagram per focus state with
/// the focused entity's styles baked in statically.
///
/// # Durations
///
/// The focus-independent pipeline steps (input merge, IR structure, and taffy
/// layout) are computed once and shared across every focus, so their durations
/// in [`DiagramGenerated`] are identical across all entries. The
/// `ir_diagram_map_duration` additionally includes the per-focus tailwind class
/// pass, and the SVG elements / SVG durations are measured per focus.
#[derive(Clone, Debug, PartialEq)]
pub struct DiagramFocusGenerated {
    /// The focus state this diagram was generated for.
    pub focus: DiagramFocus<'static>,
    /// The generated diagram for this focus state.
    pub diagram_generated: DiagramGenerated,
}
