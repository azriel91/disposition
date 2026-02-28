//! Editor tab bar component.
//!
//! Renders the top-level tab bar and, when a Theme page is active, a nested
//! sub-tab bar.
//!
//! Supports keyboard navigation:
//!
//! - **Tab**: focus the tab bar.
//! - **Left / Right arrows**: move between tabs within the bar.
//! - **Enter / Space**: activate the focused tab.

use dioxus::{
    document,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Key, Props},
    signals::{ReadableExt, Signal, WritableExt},
};

use crate::editor_state::{EditorPage, EditorPageOrGroup};

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

/// Renders the top-level tab bar and, when a Theme page is active, a nested
/// sub-tab bar.
///
/// The tab bars use `role="tablist"` / `role="tab"` semantics and support
/// Left/Right arrow key navigation as well as Enter/Space activation.
/// Only the currently active tab participates in the document tab order
/// (`tabindex="0"`); the remaining tabs use `tabindex="-1"` and are
/// reachable via arrow keys once the bar is focused.
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
                role: "tablist",

                onkeydown: move |evt| {
                    let top_level = EditorPage::TOP_LEVEL;
                    let len = top_level.len();
                    if len == 0 {
                        return;
                    }

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

                for (tab_idx, entry) in EditorPage::TOP_LEVEL.iter().enumerate() {
                    {
                        let is_active = entry.contains(&current);
                        let css = format!(
                            "{TAB_CLASS} {}",
                            if is_active { TAB_ACTIVE } else { TAB_INACTIVE }
                        );
                        // Only the active tab is in the tab order; others use
                        // tabindex="-1" and are reachable via arrow keys.
                        let tab_index = if is_active { "0" } else { "-1" };
                        let entry_clone = entry.clone();
                        let _ = tab_idx;

                        rsx! {
                            span {
                                key: "{entry.label()}",
                                role: "tab",
                                tabindex: "{tab_index}",
                                "aria-selected": if is_active { "true" } else { "false" },
                                class: "{css}",
                                onclick: {
                                    let entry = entry_clone.clone();
                                    move |_| {
                                        editor_tab_bar_top_level_activate(active_page, &entry);
                                    }
                                },
                                onkeydown: {
                                    let entry = entry_clone.clone();
                                    move |evt| {
                                        let activate = match evt.key() {
                                            Key::Enter => true,
                                            Key::Character(ref c) if c == " " => true,
                                            _ => false,
                                        };
                                        if activate {
                                            evt.prevent_default();
                                            editor_tab_bar_top_level_activate(active_page, &entry);
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

                    for sub in EditorPage::THEME_SUB_PAGES.iter() {
                        {
                            let is_active = current == *sub;
                            let css = format!(
                                "{SUB_TAB_CLASS} {}",
                                if is_active { TAB_ACTIVE } else { TAB_INACTIVE }
                            );
                            let tab_index = if is_active { "0" } else { "-1" };
                            let sub_clone = sub.clone();

                            rsx! {
                                span {
                                    key: "{sub.label()}",
                                    role: "tab",
                                    tabindex: "{tab_index}",
                                    "aria-selected": if is_active { "true" } else { "false" },
                                    class: "{css}",
                                    onclick: {
                                        let sub_page = sub_clone.clone();
                                        move |_| {
                                            active_page.set(sub_page.clone());
                                        }
                                    },
                                    onkeydown: {
                                        let sub_page = sub_clone.clone();
                                        move |evt| {
                                            let activate = match evt.key() {
                                                Key::Enter => true,
                                                Key::Character(ref c) if c == " " => true,
                                                _ => false,
                                            };
                                            if activate {
                                                evt.prevent_default();
                                                active_page.set(sub_page.clone());
                                            }
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

/// Activates a top-level tab entry.
///
/// For a single [`EditorPage`], sets it directly. For the
/// [`ThemeGroup`](EditorPageOrGroup::ThemeGroup), only switches to
/// `ThemeStyleAliases` if the user is not already on a theme page.
fn editor_tab_bar_top_level_activate(
    mut active_page: Signal<EditorPage>,
    entry: &EditorPageOrGroup,
) {
    match entry {
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
