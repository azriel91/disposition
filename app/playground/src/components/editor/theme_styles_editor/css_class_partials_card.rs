//! A single card within the theme styles editor.
//!
//! Composes the header, style aliases, and theme attributes sub-components
//! into one card per `IdOrDefaults -> CssClassPartials` entry.
//!
//! Supports keyboard shortcuts:
//!
//! - **ArrowUp / ArrowDown**: navigate between sibling cards.
//! - **Alt+Up / Alt+Down**: move the card up or down in the list.
//! - **ArrowRight**: expand the card (when collapsed).
//! - **ArrowLeft**: collapse the card (when expanded).
//! - **Space**: toggle expand/collapse.
//! - **Enter**: expand + focus the first input inside the card.
//! - **Ctrl+Shift+K**: remove the card.
//! - **Escape**: focus the parent section / tab.
//! - **Tab / Shift+Tab** (inside a field): cycle through focusable fields
//!   within the card. Wraps from last to first / first to last.
//! - **Esc** (inside a field): return focus to the card wrapper.
//!
//! When collapsed, shows the entry key and number of attributes.

use dioxus::{
    prelude::{
        component, dioxus_core, dioxus_elements, dioxus_signals, rsx, use_context, Element, Memo,
        Props,
    },
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_ir_rt::ThemeValueSource;

use crate::components::editor::{
    common::CardComponent,
    keyboard_nav::KeyboardNav,
    reorderable::{drag_border_class, DragHandle},
    theme_styles_editor::{
        css_class_partials_card_aliases::CssClassPartialsCardAliases,
        css_class_partials_card_attrs::CssClassPartialsCardAttrs,
        css_class_partials_card_header::CssClassPartialsCardHeader,
        css_class_partials_card_summary::CssClassPartialsCardSummary, parse_id_or_defaults,
        theme_attr_entry::ThemeAttrEntry, ThemeStylesTarget,
    },
};

// === Data attribute for the card wrapper === //

/// The `data-*` attribute placed on each `CssClassPartialsCard` wrapper.
///
/// Used by [`KeyboardNav`] helpers to locate the nearest ancestor card.
pub(crate) const DATA_ATTR: &str = "data-css-card";

/// CSS classes for the "revert to base" button.
const REVERT_BTN: &str = "\
    text-xs \
    text-amber-400 \
    hover:text-amber-300 \
    cursor-pointer \
    select-none\
";

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
pub(crate) const COLLAPSED_HEADER_CLASS: &str = "\
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
    entry_count: usize,
    entry_key: String,
    style_aliases: Vec<String>,
    theme_attrs: Vec<ThemeAttrEntry>,
    value_source: ThemeValueSource,
    drag_index: Signal<Option<usize>>,
    drop_target: Signal<Option<usize>>,
    mut focus_index: Signal<Option<usize>>,
) -> Element {
    let card_state = CardComponent::state_init(entry_index, entry_count, &entry_key);
    let mut collapsed = card_state.collapsed;
    let border_class = drag_border_class(drag_index, drop_target, entry_index);
    let base_diagram: Memo<InputDiagram<'static>> = use_context();

    let alias_count = style_aliases.len();
    let attr_count = theme_attrs.len();

    // Pre-clone `target` for closures that need their own copy, so
    // the final use inside the `rsx!` block can move the original.
    let target_for_keydown = target.clone();
    let target_for_keydown_down = target.clone();
    let target_for_keydown_remove = target.clone();
    let target_for_drop = target.clone();
    let target_for_summary = target.clone();
    let target_for_header = target.clone();
    let target_for_aliases = target.clone();
    let target_for_revert = target.clone();

    rsx! {
        div {
            class: "{CSS_CARD_CLASS} {border_class}",
            tabindex: "0",
            draggable: "true",
            "data-css-card": "true",
            "data-input-diagram-field": "{entry_key}",

            // === Card-level keyboard shortcuts === //
            onkeydown: {
                let entry_key = entry_key.clone();
                CardComponent::card_onkeydown(
                    DATA_ATTR,
                    card_state,
                    move || {
                        let target = target_for_keydown.clone();
                        let base = base_diagram.read();
                        target.entry_move(&base, input_diagram, entry_index, entry_index - 1);
                        focus_index.set(Some(entry_index - 1));
                    },
                    move || {
                        let target = target_for_keydown_down.clone();
                        let base = base_diagram.read();
                        target.entry_move(&base, input_diagram, entry_index, entry_index + 1);
                        focus_index.set(Some(entry_index + 1));
                    },
                    move || {
                        let target = target_for_keydown_remove.clone();
                        if let Some(parsed) = parse_id_or_defaults(&entry_key) {
                            let mut diagram = input_diagram.write();
                            if let Some(styles) = target.write_mut(&mut diagram) {
                                styles.remove(&parsed);
                            }
                        }
                    },
                    None,
                )
            },

            // === Drag-and-drop === //
            ondragstart: move |_| {
                drag_index.set(Some(entry_index));
            },
            ondragover: move |evt| {
                evt.prevent_default();
                drop_target.set(Some(entry_index));
            },
            ondrop: move |evt: dioxus::events::DragEvent| {
                evt.prevent_default();
                if let Some(from) = *drag_index.read()
                    && from != entry_index
                {
                    let base = base_diagram.read();
                    target_for_drop.entry_move(&base, input_diagram, from, entry_index);
                }
                drag_index.set(None);
                drop_target.set(None);
            },
            ondragend: move |_| {
                drag_index.set(None);
                drop_target.set(None);
            },

            if *collapsed.read() {
                // === Collapsed summary === //
                CssClassPartialsCardSummary {
                    input_diagram,
                    target: target_for_summary,
                    entry_key: entry_key.clone(),
                    alias_count,
                    attr_count,
                    value_source,
                    collapsed,
                }
            } else {
                // === Expanded content === //

                // Collapse toggle + drag handle
                div {
                    class: "flex flex-row items-center gap-1 cursor-pointer select-none mb-1",
                    onclick: move |_| collapsed.set(true),

                    DragHandle {}

                    span {
                        class: "text-gray-500 text-xs rotate-90 inline-block",
                        ">"
                    }
                    span {
                        class: "text-xs text-gray-500",
                        "Collapse"
                    }
                }

                // === Value source indicator === //
                if value_source == ThemeValueSource::UserInput {
                    // Show "Revert to base" button
                    div {
                        class: "flex flex-row items-center gap-2 text-xs",
                        span {
                            class: "text-amber-400",
                            "Overrides base styles"
                        }
                        button {
                            class: REVERT_BTN,
                            tabindex: "0",
                            onclick: {
                                let entry_key = entry_key.clone();
                                let target = target_for_revert.clone();
                                move |_| {
                                    if let Some(parsed) = parse_id_or_defaults(&entry_key) {
                                        let mut diagram = input_diagram.write();
                                        if let Some(styles) = target.write_mut(&mut diagram) {
                                            styles.remove(&parsed);
                                        }
                                    }
                                }
                            },
                            onkeydown: move |evt| {
                                css_card_field_keydown(evt);
                            },
                            "Revert to base"
                        }
                    }
                } else {
                    // BaseDiagram source — show read-only indicator
                    div {
                        class: "text-xs text-gray-500 italic",
                        "From disposition's base styles"
                    }
                }

                // === Header row: key + remove === //
                CssClassPartialsCardHeader {
                    input_diagram,
                    target: target_for_header,
                    entry_key: entry_key.clone(),
                }

                // === Style aliases applied === //
                CssClassPartialsCardAliases {
                    input_diagram,
                    target: target_for_aliases,
                    entry_key: entry_key.clone(),
                    style_aliases,
                }

                // === Theme attributes (partials map) === //
                CssClassPartialsCardAttrs {
                    input_diagram,
                    target,
                    entry_key: entry_key.clone(),
                    theme_attrs,
                }
            }
        }
    }
}

/// Shared `onkeydown` handler for inputs, selects, checkboxes, and remove
/// buttons inside a `CssClassPartialsCard`.
///
/// Delegates to [`KeyboardNav::field_keydown`] with this card's
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
    KeyboardNav::field_keydown(evt, DATA_ATTR);
}
