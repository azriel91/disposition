//! Editor tab bar component.
//!
//! Renders the top-level tab bar and, when a Things or Theme page is active,
//! a nested sub-tab bar.
//!
//! Supports keyboard navigation:
//!
//! - **Tab**: focus the tab bar.
//! - **Left / Right arrows**: move between tabs within the bar.
//! - **Enter / Space**: activate the focused tab.

use dioxus::{
    document,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Key, Props},
    signals::{ReadableExt, Signal},
};

use crate::editor_state::EditorPage;

use self::{
    editor_tab_bar_tabs::EditorTabBarTabs, editor_tab_bar_theme::EditorTabBarTheme,
    editor_tab_bar_thing::EditorTabBarThing, editor_tab_bar_thing_pages::EditorTabBarThingPages,
};

mod editor_tab_bar_tabs;
mod editor_tab_bar_theme;
mod editor_tab_bar_theme_pages;
mod editor_tab_bar_thing;
mod editor_tab_bar_thing_pages;

/// CSS classes for top-level editor page tabs.
const TAB_CLASS: &str = "\
    select-none \
    px-3 py-1.5 \
    text-sm \
    font-semibold \
    rounded-t \
    transition-colors \
    duration-150 \
    focus:outline-none \
    focus:ring-1 \
    focus:ring-blue-400\
";

const TAB_ACTIVE: &str = "text-blue-400 border-b-2 border-blue-400";
const TAB_INACTIVE: &str = "text-gray-400 hover:text-gray-200 cursor-pointer";

/// CSS classes for theme sub-tabs (smaller, nested).
const SUB_TAB_CLASS: &str = "\
    select-none \
    px-2 py-1 \
    text-xs \
    font-semibold \
    rounded-t \
    transition-colors \
    duration-150 \
    focus:outline-none \
    focus:ring-1 \
    focus:ring-blue-400\
";

/// Renders the top-level tab bar and, when a Things or Theme page is
/// active, a nested sub-tab bar.
///
/// The tab bars use `role="tablist"` / `role="tab"` semantics and support
/// Left/Right arrow key navigation as well as Enter/Space activation.
/// Only the currently active tab participates in the document tab order
/// (`tabindex="0"`); the remaining tabs use `tabindex="-1"` and are
/// reachable via arrow keys once the bar is focused.
#[component]
pub fn EditorTabBar(active_page: Signal<EditorPage>) -> Element {
    let current_page = active_page.read().clone();

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
                role: "tablist",

                onkeydown: move |evt| {
                    match evt.key() {
                        Key::ArrowLeft => {
                            evt.prevent_default();
                            // Move focus to the previous tab sibling.
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
                            // Move focus to the next tab sibling.
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

                EditorTabBarTabs { active_page }
            }

            // === Things sub-tabs (only visible when a Things page is active) === //
            match current_page {
                EditorPage::Thing(_) => rsx! { EditorTabBarThing { active_page } },
                EditorPage::Theme(_) => rsx! { EditorTabBarTheme { active_page } },
                EditorPage::ThingLayout |
                EditorPage::ThingDependencies |
                EditorPage::ThingInteractions |
                EditorPage::Processes |
                EditorPage::Tags |
                EditorPage::EntityTypes |
                EditorPage::RenderOptions |
                EditorPage::Text => rsx! {},
            }
        }
    }
}
