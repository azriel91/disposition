use dioxus::{
    prelude::{
        component, dioxus_core, dioxus_elements, dioxus_signals, document, rsx, Element, Key, Props,
    },
    signals::Signal,
};

use crate::{
    components::disposition_editor::editor_tab_bar::editor_tab_bar_entity_pages::EditorTabBarEntityPages,
    editor_state::EditorPage,
};

#[component]
pub(crate) fn EditorTabBarEntity(active_page: Signal<EditorPage>) -> Element {
    rsx! {
        div {
            class: "
                flex
                flex-row
                flex-wrap
                gap-1
                border-b
                border-gray-700
                mb-1
                pl-2
            ",
            role: "tablist",

            onkeydown: move |evt| {
                match evt.key() {
                    Key::ArrowLeft => {
                        evt.prevent_default();
                        document::eval(
                            "(() => {\
                                let el = document.activeElement;\
                                if (!el || el.getAttribute('role') !== 'tab') return;\
                                let prev = el.previousElementSibling;\
                                if (prev) prev.focus();\
                                else { let last = el.parentElement?.lastElementChild; if (last) last.focus(); }\
                            })()"
                        );
                    }
                    Key::ArrowRight => {
                        evt.prevent_default();
                        document::eval(
                            "(() => {\
                                let el = document.activeElement;\
                                if (!el || el.getAttribute('role') !== 'tab') return;\
                                let next = el.nextElementSibling;\
                                if (next) next.focus();\
                                else { let first = el.parentElement?.firstElementChild; if (first) first.focus(); }\
                            })()"
                        );
                    }
                    _ => {}
                }
            },

            EditorTabBarEntityPages { active_page }
        }
    }
}
