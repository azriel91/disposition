use crate::{
    process::{ProcessId, ProcessStepId},
    tag::TagId,
};

/// Identifies which entity is "focused" when generating a diagram.
///
/// In `DiagramGenerator`'s per-process-step-or-tag mode, one diagram is
/// generated for each `DiagramFocus`, with the focused entity's styles baked in
/// statically (instead of being toggled interactively via CSS `:focus-within`).
///
/// # Examples
///
/// ```rust
/// use disposition_input_model::{process::ProcessId, DiagramFocus};
///
/// let diagram_focus = DiagramFocus::Process(ProcessId::new("deploy").unwrap());
///
/// assert_eq!(
///     diagram_focus,
///     DiagramFocus::Process(ProcessId::new("deploy").unwrap())
/// );
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiagramFocus<'id> {
    /// Nothing is focused -- entities use their base styles.
    None,
    /// A process is focused -- its steps are revealed (expanded).
    Process(ProcessId<'id>),
    /// A process step is focused -- its process is expanded, and the things and
    /// edges it interacts with are highlighted.
    ProcessStep {
        /// The process the focused step belongs to.
        process_id: ProcessId<'id>,
        /// The focused process step.
        process_step_id: ProcessStepId<'id>,
    },
    /// A tag is focused -- things in the tag are highlighted, others dimmed.
    Tag(TagId<'id>),
}

impl<'id> DiagramFocus<'id> {
    /// Converts this focus into an owned `'static` value, cloning any borrowed
    /// IDs.
    pub fn into_static(self) -> DiagramFocus<'static> {
        match self {
            DiagramFocus::None => DiagramFocus::None,
            DiagramFocus::Process(process_id) => {
                DiagramFocus::Process(ProcessId::from(process_id.into_inner().into_static()))
            }
            DiagramFocus::ProcessStep {
                process_id,
                process_step_id,
            } => DiagramFocus::ProcessStep {
                process_id: ProcessId::from(process_id.into_inner().into_static()),
                process_step_id: ProcessStepId::from(process_step_id.into_inner().into_static()),
            },
            DiagramFocus::Tag(tag_id) => {
                DiagramFocus::Tag(TagId::from(tag_id.into_inner().into_static()))
            }
        }
    }
}
