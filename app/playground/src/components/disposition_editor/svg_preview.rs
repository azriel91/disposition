//! SVG preview component for the disposition editor.
//!
//! Renders the generated SVG diagram with share, copy, and expand buttons.
//! When expanded, the SVG fills the entire viewport.

use dioxus::{
    document,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Memo, ReadableExt, Signal},
};

use crate::components::disposition_editor::{CopyButton, ShareButton};

use self::{collapse_button::CollapseButton, expand_button::ExpandButton, svg_div::SvgDiv};

mod collapse_button;
mod expand_button;
mod svg_div;

/// CSS classes for the expand / collapse toggle button.
const BTN_CLASS: &str = "\
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
    focus:ring-offset-gray-800\
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
                SvgDiv {
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
            SvgDiv {
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
