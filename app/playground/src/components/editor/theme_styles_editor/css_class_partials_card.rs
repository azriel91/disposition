//! A single card within the theme styles editor.
//!
//! Composes the header, style aliases, and theme attributes sub-components
//! into one card per `IdOrDefaults -> CssClassPartials` entry.
//!
//! Supports keyboard shortcuts:
//!
//! - **ArrowUp / ArrowDown**: navigate between sibling cards.
//! - **ArrowRight**: expand the card (when collapsed).
//! - **ArrowLeft**: collapse the card (when expanded).
//! - **Space**: toggle expand/collapse.
//! - **Enter**: expand + focus the first input inside the card.
//! - **Escape**: focus the parent section / tab.
//! - **Tab / Shift+Tab** (inside a field): cycle through focusable fields
//!   within the card. Wraps from last to first / first to last.
//! - **Esc** (inside a field): return focus to the card wrapper.
//!
//! When collapsed, shows the entry key and number of attributes.

use dioxus::{
    hooks::use_signal,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::InputDiagram;

use crate::components::editor::{
    keyboard_nav::{self, CardKeyAction},
    theme_styles_editor::{
        css_class_partials_card_aliases::CssClassPartialsCardAliases,
        css_class_partials_card_attrs::CssClassPartialsCardAttrs,
        css_class_partials_card_header::CssClassPartialsCardHeader,
        theme_attr_entry::ThemeAttrEntry, ThemeStylesTarget,
    },
};

// === Data attribute for the card wrapper === //

/// The `data-*` attribute placed on each `CssClassPartialsCard` wrapper.
///
/// Used by [`keyboard_nav`] helpers to locate the nearest ancestor card.
pub(crate) const DATA_ATTR: &str = "data-css-card";

// === CSS === //

/// CSS classes for the focusable card wrapper.
///
/// Extends the standard card styling with focus ring and transitions.
const CSS_CARD_CLASS: &str = "\
    rounded-lg \
    border \
    border-gray-700 \
    bg-gray-900 \
    p-3 \
    mb-2 \
    flex \
    flex-col \
    gap-2 \
    focus:outline-none \
    focus:ring-1 \
    focus:ring-blue-400 \
    transition-all \
    duration-150\
";

/// CSS classes for the collapsed summary header.
const COLLAPSED_HEADER_CLASS: &str = "\
    flex \
    flex-row \
    items-center \
    gap-3 \
    cursor-pointer \
    select-none\
";

// === CssClassPartialsCard === //

/// A single card within the [`ThemeStylesEditor`].
///
/// Composes three sections:
///
/// 1. **Header** -- key selector (select for built-ins, text for custom IDs)
///    and a remove button.
/// 2. **Style aliases** -- list of applied aliases with remove buttons + add.
/// 3. **Theme attributes** -- key/value rows with remove buttons + add.
///
/// [`ThemeStylesEditor`]: super::ThemeStylesEditor
#[component]
pub fn CssClassPartialsCard(
    input_diagram: Signal<InputDiagram<'static>>,
    target: ThemeStylesTarget,
    entry_index: usize,
    entry_key: String,
    style_aliases: Vec<String>,
    theme_attrs: Vec<ThemeAttrEntry>,
) -> Element {
    let mut collapsed = use_signal(|| true);

    let alias_count = style_aliases.len();
    let attr_count = theme_attrs.len();
    let alias_suffix = if alias_count != 1 { "es" } else { "" };
    let attr_suffix = if attr_count != 1 { "s" } else { "" };

    rsx! {
        div {
            class: CSS_CARD_CLASS,
            tabindex: "0",
            "data-css-card": "true",

            // === Card-level keyboard shortcuts === //
            onkeydown: move |evt| {
                let action = keyboard_nav::card_keydown(evt, DATA_ATTR);
                match action {
                    CardKeyAction::Collapse => collapsed.set(true),
                    CardKeyAction::Expand => collapsed.set(false),
                    CardKeyAction::Toggle => {
                        let is_collapsed = *collapsed.read();
                        collapsed.set(!is_collapsed);
                    }
                    CardKeyAction::EnterEdit => collapsed.set(false),
                    CardKeyAction::None => {}
                }
            },

            if *collapsed.read() {
                // === Collapsed summary === //
                div {
                    class: COLLAPSED_HEADER_CLASS,
                    onclick: move |_| collapsed.set(false),

                    // Expand chevron
                    span {
                        class: "text-gray-500 text-xs",
                        ">"
                    }

                    span {
                        class: "text-sm font-mono text-blue-400",
                        "{entry_key}"
                    }

                    span {
                        class: "text-xs text-gray-500",
                        "({alias_count} alias{alias_suffix}, {attr_count} attr{attr_suffix})"
                    }
                }
            } else {
                // === Expanded content === //

                // Collapse toggle
                div {
                    class: "flex flex-row items-center gap-1 cursor-pointer select-none mb-1",
                    onclick: move |_| collapsed.set(true),

                    span {
                        class: "text-gray-500 text-xs rotate-90 inline-block",
                        ">"
                    }
                    span {
                        class: "text-xs text-gray-500",
                        "Collapse"
                    }
                }

                // === Header row: key + remove === //
                CssClassPartialsCardHeader {
                    input_diagram,
                    target: target.clone(),
                    entry_key: entry_key.clone(),
                }

                // === Style aliases applied === //
                CssClassPartialsCardAliases {
                    input_diagram,
                    target: target.clone(),
                    entry_key: entry_key.clone(),
                    style_aliases,
                }

                // === Theme attributes (partials map) === //
                CssClassPartialsCardAttrs {
                    input_diagram,
                    target,
                    entry_key,
                    theme_attrs,
                }
            }
        }
    }
}

/// Shared `onkeydown` handler for inputs, selects, checkboxes, and remove
/// buttons inside a `CssClassPartialsCard`.
///
/// Delegates to [`keyboard_nav::field_keydown`] with this card's
/// [`DATA_ATTR`].
///
/// - **Escape**: return focus to the parent `CssClassPartialsCard`.
/// - **Tab / Shift+Tab**: cycle through focusable fields within the card. Wraps
///   from last to first / first to last.
/// - **Enter**: stop propagation so the card-level handler does not fire.
/// - **Arrow keys**: stop propagation (allows cursor movement in text inputs
///   and select navigation).
/// - **Space**: stop propagation (prevents parent collapse toggle).
pub(crate) fn css_card_field_keydown(evt: dioxus::events::KeyboardEvent) {
    keyboard_nav::field_keydown(evt, DATA_ATTR);
}
