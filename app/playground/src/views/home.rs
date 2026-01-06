use crate::components::DispositionEditor;
use dioxus::prelude::{component, dioxus_core, dioxus_signals, rsx, Element};

/// The Home page component that will be rendered when the current route is
/// `[Route::Home]`
#[component]
pub fn Home() -> Element {
    rsx! {
        DispositionEditor {}
    }
}
