use disposition_input_model::DiagramFocus;

/// How focus-dependent tailwind classes should be emitted.
///
/// Focus (highlighting a process, process step, or tag) can either be toggled
/// interactively in the browser via CSS `:focus-within`, or baked statically
/// into a diagram generated specifically for one focus state.
#[derive(Clone, Copy, Debug)]
pub(crate) enum TailwindFocusMode<'a, 'id> {
    /// Emit interactive `peer/...` and `group-has-[...]` classes so focus is
    /// toggled at render time. This is the behaviour for a single diagram that
    /// covers all focus states.
    Interactive,
    /// Bake the resolved styles for `active` directly (no `peer` / `group-has`
    /// classes), so the generated diagram statically shows that focus state.
    Baked {
        /// The focus state whose styles are baked in.
        active: &'a DiagramFocus<'id>,
    },
}
