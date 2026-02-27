use crate::components::DispositionEditor;
use dioxus::{
    prelude::{component, dioxus_core, dioxus_signals, rsx, Element, Props, ReadableExt},
    signals::ReadSignal,
};

use crate::editor_state::EditorState;

/// The Home page component that will be rendered when the current route is
/// [`Route::Home`]
#[component]
pub fn Home(editor_state: ReadSignal<EditorState>) -> Element {
    rsx! {
        DispositionEditor { editor_state }
    }
}
