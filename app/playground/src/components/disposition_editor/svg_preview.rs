use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Memo, Signal},
};

use crate::components::disposition_editor::{CopyButton, ShareButton};

#[component]
pub fn SvgPreview(svg: Memo<String>, show_share_modal: Signal<bool>) -> Element {
    rsx! {
        div {
            class: "flex-1 flex flex-col",
            div {
                class: "\
                    flex \
                    justify-end \
                    gap-1\
                ",
                ShareButton { show_share_modal }
                CopyButton { text_to_copy: svg.clone() }
            },
            object {
                class: "
                    flex-1
                ",
                r#type: "image/svg+xml",
                data: format!("data:image/svg+xml,{}", urlencoding::encode(svg().as_str())),
            }
        }
    }
}
