//! Help tooltip component for the thing layout page.
//!
//! Renders a circled question-mark button that toggles a dismissable tooltip
//! listing all keyboard shortcuts and drag-and-drop instructions available
//! in the thing layout editor.

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};

/// CSS classes for the help toggle button (circled question mark).
const HELP_BTN: &str = "\
    w-5 \
    h-5 \
    flex \
    items-center \
    justify-center \
    rounded-full \
    border \
    border-gray-600 \
    text-gray-400 \
    hover:text-gray-200 \
    hover:border-gray-400 \
    cursor-pointer \
    text-xs \
    font-bold \
    select-none \
    leading-none \
    shrink-0\
";

/// CSS classes for the active (open) state of the help button.
const HELP_BTN_ACTIVE: &str = "\
    w-5 \
    h-5 \
    flex \
    items-center \
    justify-center \
    rounded-full \
    border \
    border-blue-400 \
    text-blue-400 \
    cursor-pointer \
    text-xs \
    font-bold \
    select-none \
    leading-none \
    shrink-0\
";

/// CSS classes for the tooltip container.
const TOOLTIP_CLASS: &str = "\
    absolute \
    right-0 \
    top-6 \
    z-10 \
    w-72 \
    rounded-lg \
    border \
    border-gray-600 \
    bg-gray-800 \
    p-3 \
    shadow-lg \
    text-xs \
    text-gray-300\
";

/// CSS classes for each shortcut row inside the tooltip.
const SHORTCUT_ROW: &str = "flex flex-row items-start gap-2 py-0.5";

/// CSS classes for the keyboard shortcut key badge.
const KBD_CLASS: &str = "\
    inline-block \
    rounded \
    border \
    border-gray-500 \
    bg-gray-700 \
    px-1 \
    py-px \
    font-mono \
    text-gray-200 \
    text-[10px] \
    leading-tight \
    whitespace-nowrap\
";

/// A help button with a dismissable tooltip showing keyboard shortcuts and
/// interaction hints for the thing layout editor.
///
/// # Props
///
/// * `show_help`: signal controlling visibility of the tooltip panel.
#[component]
pub fn HelpTooltip(show_help: Signal<bool>) -> Element {
    let is_open = *show_help.read();
    let btn_class = if is_open { HELP_BTN_ACTIVE } else { HELP_BTN };

    rsx! {
        div {
            class: "relative",

            // === Toggle button === //
            span {
                class: "{btn_class}",
                title: if is_open { "Hide keyboard shortcuts" } else { "Show keyboard shortcuts" },
                onclick: move |_| {
                    let current = *show_help.read();
                    show_help.set(!current);
                },
                "?"
            }

            // === Tooltip panel === //
            if is_open {
                div {
                    class: TOOLTIP_CLASS,

                    // Header with dismiss button.
                    div {
                        class: "flex flex-row items-center mb-2",
                        span {
                            class: "flex-1 font-bold text-gray-200 text-sm",
                            "Keyboard Shortcuts"
                        }
                        span {
                            class: "\
                                text-gray-500 \
                                hover:text-gray-300 \
                                cursor-pointer \
                                text-xs \
                                px-1 \
                                select-none\
                            ",
                            onclick: move |_| {
                                show_help.set(false);
                            },
                            "✕"
                        }
                    }

                    // === Shortcut entries === //
                    div {
                        class: "flex flex-col gap-1",

                        // Move up
                        div {
                            class: SHORTCUT_ROW,
                            span { class: KBD_CLASS, "Alt + Up" }
                            span { "Move entry up. If already first sibling, becomes sibling of parent." }
                        }

                        // Move down
                        div {
                            class: SHORTCUT_ROW,
                            span { class: KBD_CLASS, "Alt + Down" }
                            span { "Move entry down. If already last sibling, becomes sibling of parent." }
                        }

                        // Indent
                        div {
                            class: SHORTCUT_ROW,
                            span { class: KBD_CLASS, "Tab" }
                            span { "Indent -- become a child of the previous sibling." }
                        }

                        // Outdent
                        div {
                            class: SHORTCUT_ROW,
                            span { class: KBD_CLASS, "Shift + Tab" }
                            span { "Outdent -- become a sibling of the parent." }
                        }

                        // Separator
                        div {
                            class: "border-t border-gray-700 my-1",
                        }

                        // Drag and drop
                        div {
                            class: SHORTCUT_ROW,
                            span { class: KBD_CLASS, "Drag" }
                            span { "Drag the handle to reorder. The entry adopts the depth of the drop target." }
                        }

                        // Focus hint
                        div {
                            class: "text-gray-500 italic mt-1",
                            "Click a row to focus it before using keyboard shortcuts."
                        }
                    }
                }
            }
        }
    }
}
