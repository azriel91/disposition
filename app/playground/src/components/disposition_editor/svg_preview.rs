//! SVG preview component for the disposition editor.
//!
//! Renders the generated SVG diagram with share, copy, and expand buttons.
//! When expanded, the SVG fills the entire viewport.

use dioxus::{
    document,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Memo, ReadableExt, Signal, WritableExt},
};

use crate::components::disposition_editor::{CopyButton, ShareButton};

/// CSS classes for the expand / collapse toggle button.
const EXPAND_BTN_CLASS: &str = "\
    flex-none \
    flex \
    justify-center \
    items-center \
    p-2 \
    text-gray-200 \
    rounded-lg \
    bg-gray-800 \
    border-gray-600 \
    hover:bg-gray-700 \
    active:bg-gray-800 \
    focus:outline-none \
    focus:ring-2 \
    focus:ring-blue-600 \
    focus:ring-offset-2 \
    focus:ring-offset-gray-100\
";

#[component]
pub fn SvgPreview(
    svg: Memo<String>,
    show_share_modal: Signal<bool>,
    svg_preview_expanded: Signal<bool>,
) -> Element {
    let is_expanded = *svg_preview_expanded.read();

    // When expanded, render a full-viewport overlay.
    if is_expanded {
        return rsx! {
            div {
                class: "\
                    fixed \
                    inset-0 \
                    z-[100] \
                    flex \
                    flex-col \
                    bg-gray-900\
                ",
                tabindex: "-1",
                "data-svg-expanded": "",

                // Auto-focus the overlay so the global JS keydown listener
                // (registered in `DispositionEditor`) sees key events while
                // the overlay is open. We do NOT add a Dioxus `onkeydown`
                // here because the global listener already handles `f` and
                // `Escape`; having both would double-fire and toggle the
                // state back.
                onmounted: move |_| {
                    document::eval(
                        "requestAnimationFrame(() => {\
                            var el = document.querySelector('[data-svg-expanded]');\
                            if (el) el.focus();\
                        })"
                    );
                },

                // === Top bar with restore button === //
                div {
                    class: "\
                        flex \
                        justify-end \
                        gap-1 \
                        p-2\
                    ",
                    ShareButton { show_share_modal }
                    CopyButton { text_to_copy: svg }
                    CollapseButton { svg_preview_expanded }
                }

                // === SVG fills remaining space === //
                SvgScrollable {
                    class: "
                        flex-1 \
                        fit-content \
                        overflow-auto\
                    ",
                    svg,
                }
            }
        };
    }

    // Normal (non-expanded) view.
    rsx! {
        div {
            class: "\
                flex-1 \
                flex \
                flex-col \
                [&>*]:shrink \
                overflow-auto\
            ",
            div {
                class: "\
                    flex \
                    justify-end \
                    gap-1\
                ",
                ShareButton { show_share_modal }
                CopyButton { text_to_copy: svg }
                ExpandButton { svg_preview_expanded }
            },
            SvgScrollable {
                class: "
                    flex-1 \
                    fit-content \
                    overflow-auto\
                ",
                svg,
            }
        }
    }
}

#[component]
fn SvgScrollable(class: &'static str, svg: Memo<String>) -> Element {
    rsx! {
        div {
            class,
            dangerous_inner_html: svg(),
        }
    }
}

#[component]
fn ExpandButton(svg_preview_expanded: Signal<bool>) -> Element {
    rsx! {
        button {
            class: EXPAND_BTN_CLASS,
            tabindex: "0",
            title: "Expand SVG preview (f)",
            onclick: move |_| {
                svg_preview_expanded.set(true);
            },
            ExpandIcon {}
        }
    }
}

/// Expand icon: arrows pointing outward to the four corners.
#[component]
fn ExpandIcon() -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            class: "h-5 w-5",
            width: "24",
            height: "24",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            // Top-left corner.
            polyline { points: "8 3 3 3 3 8" }
            line { x1: "3", y1: "3", x2: "10", y2: "10" }

            // Top-right corner.
            polyline { points: "16 3 21 3 21 8" }
            line { x1: "21", y1: "3", x2: "14", y2: "10" }

            // Bottom-left corner.
            polyline { points: "8 21 3 21 3 16" }
            line { x1: "3", y1: "21", x2: "10", y2: "14" }

            // Bottom-right corner.
            polyline { points: "16 21 21 21 21 16" }
            line { x1: "21", y1: "21", x2: "14", y2: "14" }
        }
    }
}

#[component]
fn CollapseButton(svg_preview_expanded: Signal<bool>) -> Element {
    rsx! {
        button {
            class: EXPAND_BTN_CLASS,
            tabindex: "0",
            title: "Restore editor (Escape / f)",
            onclick: move |_| {
                svg_preview_expanded.set(false);
            },
            CollapseIcon {}
        }
    }
}

/// Collapse icon: arrows pointing inward from the four corners.
#[component]
fn CollapseIcon() -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            class: "h-5 w-5",
            width: "24",
            height: "24",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",

            // From top-left corner inward.
            polyline { points: "4 14 10 14 10 20" }
            line { x1: "3", y1: "21", x2: "10", y2: "14" }

            // From top-right corner inward.
            polyline { points: "20 14 14 14 14 20" }
            line { x1: "21", y1: "21", x2: "14", y2: "14" }

            // From bottom-left corner inward.
            polyline { points: "4 10 10 10 10 4" }
            line { x1: "3", y1: "3", x2: "10", y2: "10" }

            // From bottom-right corner inward.
            polyline { points: "20 10 14 10 14 4" }
            line { x1: "21", y1: "3", x2: "14", y2: "10" }
        }
    }
}
