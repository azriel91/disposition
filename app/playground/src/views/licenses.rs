use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element},
    signals::Signal,
};

use crate::hooks::use_dark_mode;

/// Page to show licenses of all transitive dependencies of `disposition`.
#[component]
pub fn Licenses() -> Element {
    let is_dark: Signal<bool> = use_dark_mode();
    rsx! {
        iframe {
            class: "w-full h-[90dvh]",
            class: if is_dark() { "scheme-dark" } else { "scheme-light" },
            srcdoc: include_str!("../../assets/licenses.html")
        }
    }
}
