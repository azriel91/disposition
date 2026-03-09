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
    signals::{ReadableExt, Signal, WritableExt},
};

use crate::editor_state::{EditorPage, EditorPageTheme, EditorPageThing};

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
    let current = active_page.read().clone();
    let top_level = EditorPage::top_level_pages();

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

                for (tab_idx, entry) in top_level.iter().enumerate() {
                    {
                        let is_active = entry.same_top_level(&current);
                        let css = format!(
                            "{TAB_CLASS} {}",
                            if is_active { TAB_ACTIVE } else { TAB_INACTIVE }
                        );
                        // Only the active tab is in the tab order; others use
                        // tabindex="-1" and are reachable via arrow keys.
                        let tab_index = if is_active { "0" } else { "-1" };
                        let entry_clone = entry.clone();
                        let tab_idx_str = tab_idx.to_string();

                        rsx! {
                            span {
                                key: "{entry.top_level_label()}",
                                role: "tab",
                                tabindex: "{tab_index}",
                                "aria-selected": if is_active { "true" } else { "false" },
                                "data-top-level-index": "{tab_idx_str}",
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
                                "{entry.top_level_label()}"
                            }
                        }
                    }
                }
            }

            // === Things sub-tabs (only visible when a Things page is active) === //
            if current.is_thing() {
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

                    for sub in enum_iterator::all::<EditorPageThing>() {
                        {
                            let sub_page = EditorPage::Thing(sub.clone());
                            let is_active = current == sub_page;
                            let css = format!(
                                "{SUB_TAB_CLASS} {}",
                                if is_active { TAB_ACTIVE } else { TAB_INACTIVE }
                            );
                            let tab_index = if is_active { "0" } else { "-1" };
                            let sub_page_click = sub_page.clone();
                            let sub_page_key = sub_page.clone();

                            rsx! {
                                span {
                                    key: "{sub.label()}",
                                    role: "tab",
                                    tabindex: "{tab_index}",
                                    "aria-selected": if is_active { "true" } else { "false" },
                                    class: "{css}",
                                    onclick: {
                                        let page = sub_page_click.clone();
                                        move |_| {
                                            active_page.set(page.clone());
                                        }
                                    },
                                    onkeydown: {
                                        let page = sub_page_key.clone();
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
                                    "{sub.label()}"
                                }
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

                    for sub in enum_iterator::all::<EditorPageTheme>() {
                        {
                            let sub_page = EditorPage::Theme(sub.clone());
                            let is_active = current == sub_page;
                            let css = format!(
                                "{SUB_TAB_CLASS} {}",
                                if is_active { TAB_ACTIVE } else { TAB_INACTIVE }
                            );
                            let tab_index = if is_active { "0" } else { "-1" };
                            let sub_page_click = sub_page.clone();
                            let sub_page_key = sub_page.clone();

                            rsx! {
                                span {
                                    key: "{sub.label()}",
                                    role: "tab",
                                    tabindex: "{tab_index}",
                                    "aria-selected": if is_active { "true" } else { "false" },
                                    class: "{css}",
                                    onclick: {
                                        let page = sub_page_click.clone();
                                        move |_| {
                                            active_page.set(page.clone());
                                        }
                                    },
                                    onkeydown: {
                                        let page = sub_page_key.clone();
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
/// For grouped tabs (`Thing(_)` or `Theme(_)`), only switches to the
/// group's default sub-page if the user is not already on a page within
/// that group. This preserves the user's sub-tab selection when
/// re-clicking the same group tab.
fn editor_tab_bar_top_level_activate(mut active_page: Signal<EditorPage>, entry: &EditorPage) {
    match entry {
        EditorPage::Thing(_) => {
            // If already on a thing page, stay there;
            // otherwise default to Thing::Names.
            if !active_page.peek().is_thing() {
                active_page.set(EditorPage::Thing(EditorPageThing::default()));
            }
        }
        EditorPage::Theme(_) => {
            // If already on a theme page, stay there;
            // otherwise default to Theme::StyleAliases.
            if !active_page.peek().is_theme() {
                active_page.set(EditorPage::Theme(EditorPageTheme::default()));
            }
        }
        other => {
            active_page.set(other.clone());
        }
    }
}
