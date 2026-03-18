use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Key, Props},
    signals::{ReadableExt, Signal, WritableExt},
};

use crate::{
    components::disposition_editor::editor_tab_bar::{SUB_TAB_CLASS, TAB_ACTIVE, TAB_INACTIVE},
    editor_state::{EditorPage, EditorPageThing},
};

#[component]
pub fn EditorTabBarThingPages(active_page: Signal<EditorPage>) -> Element {
    let current_page = active_page.read().clone();

    rsx! {
        for thing_sub_page_variant in enum_iterator::all::<EditorPageThing>() {
            {
                let thing_sub_page = EditorPage::Thing(thing_sub_page_variant.clone());
                let is_active = current_page == thing_sub_page;
                let css = format!(
                    "{SUB_TAB_CLASS} {}",
                    if is_active { TAB_ACTIVE } else { TAB_INACTIVE }
                );
                let tab_index = if is_active { "0" } else { "-1" };
                let thing_sub_page_click = thing_sub_page.clone();
                let thing_sub_page_key = thing_sub_page.clone();

                rsx! {
                    span {
                        key: "{thing_sub_page_variant.label()}",
                        role: "tab",
                        tabindex: "{tab_index}",
                        "aria-selected": if is_active { "true" } else { "false" },
                        class: "{css}",
                        onclick: {
                            let page = thing_sub_page_click.clone();
                            move |_| {
                                active_page.set(page.clone());
                            }
                        },
                        onkeydown: {
                            let page = thing_sub_page_key.clone();
                            move |evt| {
                                let activate = match evt.key() {
                                    Key::Enter => true,
                                    Key::Character(ref c) if c == " " => true,
                                    _ => false,
                                };
                                if activate {
                                    evt.prevent_default();
                                    active_page.set(page.clone());
                                }
                            }
                        },
                        "{thing_sub_page_variant.label()}"
                    }
                }
            }
        }
    }
}
