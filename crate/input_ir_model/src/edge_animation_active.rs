use serde::{Deserialize, Serialize};

/// Controls when edge CSS animations are active in the rendered SVG.
///
/// This enum is passed to `TaffyToSvgElementsMapper::map` to choose whether
/// edge animations run unconditionally or only when a related process step
/// has focus.
///
/// # Variants
///
/// * `Always` – animation classes are attached directly (the current default
///   behaviour).  The edges animate continuously.
/// * `OnProcessStepFocus` – animation classes are prefixed with
///   `group-has-[#{process_step_id}:focus-within]:` so that each edge only
///   animates when one of its associated process steps is focused.
///
/// # Examples
///
/// ```rust
/// use disposition_input_ir_model::EdgeAnimationActive;
///
/// let mode = EdgeAnimationActive::default();
/// assert!(matches!(mode, EdgeAnimationActive::Always));
/// ```
#[cfg_attr(
    all(feature = "openapi", not(feature = "test")),
    derive(utoipa::ToSchema)
)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Deserialize, Serialize, Default)]
pub enum EdgeAnimationActive {
    /// Animations are always running on interaction edges.
    #[default]
    Always,
    /// Animations are only active when a related process step has
    /// `:focus-within`.
    OnProcessStepFocus,
}
