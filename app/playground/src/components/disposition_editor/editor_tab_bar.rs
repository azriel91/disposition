//! Editor tab bar component.
//!
//! Renders the top-level tab bar and, when a Theme page is active, a nested
//! sub-tab bar.

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};

use crate::editor_state::{EditorPage, EditorPageOrGroup};

/// CSS classes for top-level editor page tabs.
const TAB_CLASS: &str = "\
    cursor-pointer \
    select-none \
    px-3 py-1.5 \
    text-sm \
    font-semibold \
    rounded-t \
    transition-colors \
    duration-150\
";

const TAB_ACTIVE: &str = "text-blue-400 border-b-2 border-blue-400";
const TAB_INACTIVE: &str = "text-gray-400 hover:text-gray-200";

/// CSS classes for theme sub-tabs (smaller, nested).
const SUB_TAB_CLASS: &str = "\
    cursor-pointer \
    select-none \
    px-2 py-1 \
    text-xs \
    font-semibold \
    rounded-t \
    transition-colors \
    duration-150\
";

/// Renders the top-level tab bar and, when a Theme page is active, a nested
/// sub-tab bar.
#[component]
pub fn EditorTabBar(active_page: Signal<EditorPage>) -> Element {
    let current = active_page.read().clone();

    rsx! {
        div {
            class: "flex flex-col",

            // === Top-level tabs === //
            div {
                class: "
                    flex
                    flex-row
                    flex-wrap
                    gap-1
                    border-b
                    border-gray-700
                    mb-1
                ",

                for entry in EditorPage::TOP_LEVEL.iter() {
                    {
                        let is_active = entry.contains(&current);
                        let css = format!(
                            "{TAB_CLASS} {}",
                            if is_active { TAB_ACTIVE } else { TAB_INACTIVE }
                        );
                        let entry_clone = entry.clone();

                        rsx! {
                            span {
                                key: "{entry.label()}",
                                class: "{css}",
                                onclick: {
                                    let entry = entry_clone.clone();
                                    move |_| {
                                        match &entry {
                                            EditorPageOrGroup::Page(p) => {
                                                active_page.set(p.clone());
                                            }
                                            EditorPageOrGroup::ThemeGroup => {
                                                // If already on a theme page, stay there;
                                                // otherwise default to StyleAliases.
                                                if !active_page.peek().is_theme() {
                                                    active_page.set(EditorPage::ThemeStyleAliases);
                                                }
                                            }
                                        }
                                    }
                                },
                                "{entry.label()}"
                            }
                        }
                    }
                }
            }

            // === Theme sub-tabs (only visible when a Theme page is active) === //
            if current.is_theme() {
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

                    for sub in EditorPage::THEME_SUB_PAGES.iter() {
                        {
                            let is_active = current == *sub;
                            let css = format!(
                                "{SUB_TAB_CLASS} {}",
                                if is_active { TAB_ACTIVE } else { TAB_INACTIVE }
                            );
                            let sub_clone = sub.clone();

                            rsx! {
                                span {
                                    key: "{sub.label()}",
                                    class: "{css}",
                                    onclick: {
                                        let sub_page = sub_clone.clone();
                                        move |_| {
                                            active_page.set(sub_page.clone());
                                        }
                                    },
                                    "{sub.label()}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
