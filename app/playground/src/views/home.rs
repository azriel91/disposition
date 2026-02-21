use crate::components::DispositionEditor;
use dioxus::{
    prelude::{component, dioxus_core, dioxus_signals, rsx, Element, Props, ReadableExt},
    signals::ReadSignal,
};

/// The Home page component that will be rendered when the current route is
/// `[Route::Home]`
#[component]
pub fn Home(url_hash: ReadSignal<String>) -> Element {
    rsx! {
        DispositionEditor { url_hash }
    }
}
