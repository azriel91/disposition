//! Help tooltip component for the disposition editor.
//!
//! Renders a circled question-mark button that toggles a dismissable tooltip
//! listing all keyboard shortcuts available in the disposition editor.

use dioxus::{
    core::spawn,
    hooks::{use_effect, use_signal},
    prelude::{
        component, dioxus_core, dioxus_elements, dioxus_signals, document, rsx, Element, Props,
    },
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
    w-80 \
    rounded-lg \
    border \
    border-gray-600 \
    bg-gray-800 \
    p-3 \
    shadow-lg \
    text-xs \
    text-gray-300 \
    max-h-[80vh] \
    overflow-y-auto\
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

/// CSS classes for a section heading inside the tooltip.
const SECTION_HEADING: &str = "\
    font-semibold \
    text-gray-400 \
    uppercase \
    tracking-wide \
    text-[10px] \
    mt-2 \
    mb-0.5\
";

/// CSS classes for the divider between sections.
const DIVIDER: &str = "border-t border-gray-700 my-1";

/// Returns `"Cmd"` on macOS and `"Ctrl"` on all other platforms, detected via
/// the browser `navigator.platform` / `navigator.userAgentData` APIs.
///
/// Because this runs inside a Dioxus component (WASM), the detection is done
/// with a small inline JS expression evaluated at render time.  On non-WASM
/// builds (e.g. native preview) it always returns `"Ctrl"`.
async fn ctrl_key_label() -> &'static str {
    let is_mac = document::eval(
        "const isMac = /(Mac|iPhone|iPod|iPad)/i.test(navigator.platform || navigator.userAgentData?.platform || '') || false; dioxus.send(isMac);"
    )
    .recv::<bool>()
    .await
    .ok()
    .unwrap_or(false);

    if is_mac {
        "Cmd"
    } else {
        "Ctrl"
    }
}

/// A help button with a dismissable tooltip showing all keyboard shortcuts
/// available in the disposition editor.
///
/// Toggle visibility with `Shift + ?` or by clicking the `?` button.
///
/// # Props
///
/// * `show_help`: signal controlling visibility of the tooltip panel.
#[component]
pub fn HelpTooltip(show_help: Signal<bool>) -> Element {
    let is_open = *show_help.read();
    let btn_class = if is_open { HELP_BTN_ACTIVE } else { HELP_BTN };
    let mut ctrl = use_signal(|| "Ctrl");
    use_effect(move || {
        spawn(async move {
            let ctrl_key_label = ctrl_key_label().await;
            ctrl.set(ctrl_key_label);
        });
    });

    rsx! {
        div {
            class: "relative",

            // === Toggle button === //
            span {
                class: "{btn_class}",
                title: if is_open { "Hide keyboard shortcuts (Shift + ?)" } else { "Show keyboard shortcuts (Shift + ?)" },
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
                            "x"
                        }
                    }

                    div {
                        class: "flex flex-col gap-0.5",

                        // === Navigation === //
                        div { class: SECTION_HEADING, "Navigation" }

                        div {
                            class: SHORTCUT_ROW,
                            span { class: KBD_CLASS, "Alt + 1 .. 9" }
                            span { "Focus the numbered page tab." }
                        }
                        div {
                            class: SHORTCUT_ROW,
                            span { class: KBD_CLASS, "Tab" }
                            span { class: KBD_CLASS, "Shift + Tab" }
                            span { "Move to the next / previous field." }
                        }
                        div {
                            class: SHORTCUT_ROW,
                            span { class: KBD_CLASS, "Up" }
                            span { class: KBD_CLASS, "Down" }
                            span { "Move to the previous / next field." }
                        }
                        div {
                            class: SHORTCUT_ROW,
                            span { class: KBD_CLASS, "{ctrl} + Up" }
                            span { class: KBD_CLASS, "{ctrl} + Down" }
                            span { "Jump to the first / last field." }
                        }

                        div { class: DIVIDER }

                        // === Editing === //
                        div { class: SECTION_HEADING, "Editing" }

                        div {
                            class: SHORTCUT_ROW,
                            span { class: KBD_CLASS, "Enter" }
                            span { "Start edit mode for the focused field." }
                        }
                        div {
                            class: SHORTCUT_ROW,
                            span { class: KBD_CLASS, "Escape" }
                            span { "Stop edit mode for the focused field." }
                        }

                        div { class: DIVIDER }

                        // === Reordering === //
                        div { class: SECTION_HEADING, "Reordering Entries" }

                        div {
                            class: SHORTCUT_ROW,
                            span { class: KBD_CLASS, "Alt + Up" }
                            span { "Move the current entry up." }
                        }
                        div {
                            class: SHORTCUT_ROW,
                            span { class: KBD_CLASS, "Alt + Down" }
                            span { "Move the current entry down." }
                        }
                        div {
                            class: SHORTCUT_ROW,
                            span { class: KBD_CLASS, "Alt + Shift + Up" }
                            span { "Insert a new entry above the current entry." }
                        }
                        div {
                            class: SHORTCUT_ROW,
                            span { class: KBD_CLASS, "Alt + Shift + Down" }
                            span { "Insert a new entry below the current entry." }
                        }
                        div {
                            class: SHORTCUT_ROW,
                            span { class: KBD_CLASS, "{ctrl} + Shift + K" }
                            span { "Delete the current entry." }
                        }

                        div { class: DIVIDER }

                        // === Thing Names page === //
                        div { class: SECTION_HEADING, "Thing Names Page" }

                        div {
                            class: SHORTCUT_ROW,
                            span { class: KBD_CLASS, "Alt + Shift + D" }
                            span { "Duplicate the current Thing, including its layout, edges, and styles." }
                        }

                        div { class: DIVIDER }

                        // === Thing Layout page === //
                        div { class: SECTION_HEADING, "Thing Layout Page (edit mode)" }

                        div {
                            class: SHORTCUT_ROW,
                            span { class: KBD_CLASS, "Tab" }
                            span { "Indent -- make the current thing a child of the previous sibling." }
                        }
                        div {
                            class: SHORTCUT_ROW,
                            span { class: KBD_CLASS, "Shift + Tab" }
                            span { "Outdent -- make the current thing a sibling of its parent." }
                        }

                        div { class: DIVIDER }

                        // === Undo / Redo === //
                        div { class: SECTION_HEADING, "Undo / Redo" }

                        div {
                            class: SHORTCUT_ROW,
                            span { class: KBD_CLASS, "{ctrl} + Z" }
                            span { "Undo." }
                        }
                        div {
                            class: SHORTCUT_ROW,
                            span { class: KBD_CLASS, "{ctrl} + Y" }
                            span { class: KBD_CLASS, "{ctrl} + Shift + Z" }
                            span { "Redo." }
                        }

                        div { class: DIVIDER }

                        // === Share === //
                        div { class: SECTION_HEADING, "Share" }

                        div {
                            class: SHORTCUT_ROW,
                            span { class: KBD_CLASS, "{ctrl} + K" }
                            span { "Open the share modal with a link to the current editor state." }
                        }

                        div { class: DIVIDER }

                        // === SVG Preview === //
                        div { class: SECTION_HEADING, "SVG Preview" }

                        div {
                            class: SHORTCUT_ROW,
                            span { class: KBD_CLASS, "f" }
                            span { "Toggle expand / collapse the SVG preview to fill the page." }
                        }
                        div {
                            class: SHORTCUT_ROW,
                            span { class: KBD_CLASS, "Escape" }
                            span { "Collapse the SVG preview back to the editor layout." }
                        }

                        div { class: DIVIDER }

                        // === Help === //
                        div { class: SECTION_HEADING, "Help" }

                        div {
                            class: SHORTCUT_ROW,
                            span { class: KBD_CLASS, "Shift + ?" }
                            span { "Show / hide this keyboard shortcuts panel." }
                        }
                    }
                }
            }
        }
    }
}
