use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::Memo,
};

/// Uses `dangerous_inner_html` to render an SVG string into a div.
#[component]
pub fn SvgDiv(class: &'static str, svg: Memo<String>) -> Element {
    rsx! {
        div {
            class,
            dangerous_inner_html: svg(),
        }
    }
}
