//! Shared keyboard navigation helpers for editor cards and rows.
//!
//! Provides parameterised JavaScript snippets and `onkeydown` handler
//! functions that unify the keyboard behaviour across all editor pages.
//!
//! ## Card / Row Keyboard Behaviour
//!
//! **Entry-level** (the focusable card or row wrapper):
//!
//! - **Up / Down**: navigate to the previous / next sibling entry.
//! - **Alt+Up / Alt+Down**: reorder the entry (only for rows with drag-and-drop
//!   support -- handled by the caller, not this module).
//! - **Left**: collapse the entry (if collapsible).
//! - **Right**: expand the entry (if collapsible).
//! - **Space**: toggle collapsed state (if collapsible).
//! - **Enter**: expand (if collapsed) and focus the first interactive element
//!   inside the entry.
//! - **Escape**: focus the parent section / tab.
//!
//! **Field-level** (inputs, selects, buttons inside an entry):
//!
//! - **Tab**: cycle to the next focusable field within the same entry. On the
//!   last field, wraps to the first field.
//! - **Shift+Tab**: cycle to the previous focusable field within the same
//!   entry. On the first field, wraps to the last field.
//! - **Escape**: return focus to the parent entry wrapper.
//! - **Enter**: stop propagation (so the entry-level handler does not fire).
//! - **Arrow keys**: stop propagation (allows cursor movement in text inputs
//!   and native select navigation).
//! - **Space**: stop propagation (prevents the parent card from toggling
//!   collapse when typing in an input).

use dioxus::{
    document,
    prelude::{Key, ModifiersInteraction},
};

// === Parameterised JS snippet builders === //

/// Build a JS snippet that focuses the parent entry element identified by
/// `data_attr`.
///
/// The generated code finds the nearest ancestor with `[data_attr]` and
/// calls `.focus()` on it.
///
/// # Parameters
///
/// * `data_attr`: the `data-*` attribute on the entry wrapper, e.g.
///   `"data-edge-group-card"`, `"data-entry-id"`.
///
/// # Example
///
/// ```rust,ignore
/// let js = js_focus_parent_entry("data-edge-group-card");
/// // => "document.activeElement?.closest('[data-edge-group-card]')?.focus()"
/// ```
pub fn js_focus_parent_entry(data_attr: &str) -> String {
    format!(
        "document.activeElement\
            ?.closest('[{data_attr}]')\
            ?.focus()"
    )
}

/// Focusable-element CSS selector used inside cards/rows.
///
/// Matches `<input>`, `<select>`, `<textarea>`, `<button>`, and elements
/// with `[data-action="remove"]`.
const FOCUSABLE_SELECTOR: &str = "input, select, textarea, button, [data-action=\\\"remove\\\"]";

/// Build a JS snippet that cycles focus to the **next** focusable element
/// within the closest ancestor matching `[data_attr]`.
///
/// When on the last element, wraps to the first (true cycling).
pub fn js_tab_next_field(data_attr: &str) -> String {
    format!(
        "(() => {{\
            let el = document.activeElement;\
            if (!el) return;\
            let card = el.closest('[{data_attr}]');\
            if (!card) return;\
            let items = Array.from(card.querySelectorAll(\
                '{FOCUSABLE_SELECTOR}'\
            ));\
            let idx = items.indexOf(el);\
            if (idx < 0) return;\
            items[(idx + 1) % items.length].focus();\
        }})()"
    )
}

/// Build a JS snippet that cycles focus to the **previous** focusable
/// element within the closest ancestor matching `[data_attr]`.
///
/// When on the first element, wraps to the last (true cycling).
pub fn js_tab_prev_field(data_attr: &str) -> String {
    format!(
        "(() => {{\
            let el = document.activeElement;\
            if (!el) return;\
            let card = el.closest('[{data_attr}]');\
            if (!card) return;\
            let items = Array.from(card.querySelectorAll(\
                '{FOCUSABLE_SELECTOR}'\
            ));\
            let idx = items.indexOf(el);\
            if (idx < 0) return;\
            items[(idx - 1 + items.length) % items.length].focus();\
        }})()"
    )
}

/// Build a JS snippet that focuses the **previous** sibling element
/// matching `[data_attr]`.
pub fn js_focus_prev_entry(data_attr: &str) -> String {
    format!(
        "(() => {{\
            let el = document.activeElement;\
            if (!el) return;\
            let card = el.closest('[{data_attr}]') || el;\
            let prev = card.previousElementSibling;\
            while (prev) {{\
                if (prev.hasAttribute && prev.hasAttribute('{data_attr}')) {{\
                    prev.focus();\
                    return;\
                }}\
                prev = prev.previousElementSibling;\
            }}\
        }})()"
    )
}

/// Build a JS snippet that focuses the **next** sibling element matching
/// `[data_attr]`.
pub fn js_focus_next_entry(data_attr: &str) -> String {
    format!(
        "(() => {{\
            let el = document.activeElement;\
            if (!el) return;\
            let card = el.closest('[{data_attr}]') || el;\
            let next = card.nextElementSibling;\
            while (next) {{\
                if (next.hasAttribute && next.hasAttribute('{data_attr}')) {{\
                    next.focus();\
                    return;\
                }}\
                next = next.nextElementSibling;\
            }}\
        }})()"
    )
}

/// Build a JS snippet that focuses the first focusable element inside the
/// currently focused element (used when pressing Enter on a card/row).
pub fn js_focus_first_field() -> String {
    // Use a short timeout so the DOM can update (e.g. after expanding).
    format!(
        "setTimeout(() => {{\
            document.activeElement\
                ?.querySelector('{FOCUSABLE_SELECTOR}')\
                ?.focus();\
        }}, 0)"
    )
}

/// Build a JS snippet that, after an ID rename, focuses a sub-element
/// inside the newly created entry.
///
/// # Parameters
///
/// * `id_attr`: the `data-*` attribute that holds the entry's ID value, e.g.
///   `"data-entry-id"`, `"data-edge-group-card-id"`.
/// * `new_id`: the ID string the entry was renamed to, e.g. `"thing_1"`.
/// * `target`: which sub-element to focus.
pub fn js_rename_refocus(
    id_attr: &str,
    new_id: &str,
    target: &super::common::RenameRefocusTarget,
) -> String {
    use super::common::RenameRefocusTarget;

    // Selector for focusable fields -- unescaped version for building
    // the JS string that lives inside `querySelector`.
    let focusable_sel = "input, select, textarea, button, [data-action=\"remove\"]";

    match target {
        RenameRefocusTarget::NextField => {
            format!(
                "setTimeout(() => {{\
                    let card = document.querySelector(\
                        '[{id_attr}=\"{new_id}\"]'\
                    );\
                    if (!card) return;\
                    let items = Array.from(\
                        card.querySelectorAll(\
                            '{focusable_sel}'\
                        )\
                    );\
                    if (items.length > 1) {{\
                        items[1].focus();\
                    }} else if (items.length === 1) {{\
                        items[0].focus();\
                    }} else {{\
                        card.focus();\
                    }}\
                }}, 0)"
            )
        }
        RenameRefocusTarget::IdInput => {
            format!(
                "setTimeout(() => {{\
                    let card = document.querySelector(\
                        '[{id_attr}=\"{new_id}\"]'\
                    );\
                    if (!card) return;\
                    let input = card.querySelector('input');\
                    if (input) {{\
                        input.focus();\
                    }} else {{\
                        card.focus();\
                    }}\
                }}, 0)"
            )
        }
        RenameRefocusTarget::FocusParent => {
            format!(
                "setTimeout(() => {{\
                    let card = document.querySelector(\
                        '[{id_attr}=\"{new_id}\"]'\
                    );\
                    if (!card) return;\
                    card.focus();\
                }}, 0)"
            )
        }
    }
}

// === Shared field-level keydown handler === //

/// Shared `onkeydown` handler for interactive elements (inputs, selects,
/// textareas, buttons) inside a card or row wrapper.
///
/// The `data_attr` parameter identifies the wrapper element (e.g.
/// `"data-edge-group-card"`, `"data-entry-id"`).
///
/// ## Behaviour
///
/// - **Escape**: return focus to the parent wrapper.
/// - **Tab**: cycle to the next focusable field (wraps from last to first).
/// - **Shift+Tab**: cycle to the previous focusable field (wraps from first to
///   last).
/// - **Enter**: stop propagation so the wrapper-level handler does not fire.
/// - **Arrow keys**: stop propagation (allows cursor movement / native select
///   navigation).
/// - **Space**: stop propagation (prevents parent collapse toggle).
pub fn field_keydown(evt: dioxus::events::KeyboardEvent, data_attr: &str) {
    let shift = evt.modifiers().shift();
    match evt.key() {
        Key::Escape => {
            evt.prevent_default();
            evt.stop_propagation();
            document::eval(&js_focus_parent_entry(data_attr));
        }
        Key::Tab => {
            evt.prevent_default();
            evt.stop_propagation();
            if shift {
                document::eval(&js_tab_prev_field(data_attr));
            } else {
                document::eval(&js_tab_next_field(data_attr));
            }
        }
        Key::Enter => {
            evt.stop_propagation();
        }
        Key::ArrowUp | Key::ArrowDown | Key::ArrowLeft | Key::ArrowRight => {
            evt.stop_propagation();
        }
        Key::Character(ref c) if c == " " => {
            evt.stop_propagation();
        }
        _ => {}
    }
}

// === Shared card-level keydown handler === //

/// Outcome of the card-level keydown handler.
///
/// Callers inspect this to decide whether to toggle, expand, or collapse
/// their own `collapsed` signal. The handler already takes care of focus
/// management and `preventDefault` / `stopPropagation`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardKeyAction {
    /// No action required from the caller.
    None,
    /// The user pressed **Left** -- collapse the card.
    Collapse,
    /// The user pressed **Right** -- expand the card.
    Expand,
    /// The user pressed **Space** -- toggle collapsed state.
    Toggle,
    /// The user pressed **Enter** -- expand (if collapsed) and focus the
    /// first interactive element.
    EnterEdit,
}

/// Shared `onkeydown` handler for a collapsible card/entry wrapper.
///
/// The `data_attr` parameter identifies the wrapper element (e.g.
/// `"data-edge-group-card"`).
///
/// Returns a [`CardKeyAction`] so the caller can update its `collapsed`
/// signal accordingly. Focus management (Up/Down navigation, Enter to
/// first field) is handled internally.
pub fn card_keydown(evt: dioxus::events::KeyboardEvent, data_attr: &str) -> CardKeyAction {
    match evt.key() {
        Key::ArrowUp => {
            evt.prevent_default();
            evt.stop_propagation();
            document::eval(&js_focus_prev_entry(data_attr));
            CardKeyAction::None
        }
        Key::ArrowDown => {
            evt.prevent_default();
            evt.stop_propagation();
            document::eval(&js_focus_next_entry(data_attr));
            CardKeyAction::None
        }
        Key::ArrowLeft => {
            evt.prevent_default();
            evt.stop_propagation();
            CardKeyAction::Collapse
        }
        Key::ArrowRight => {
            evt.prevent_default();
            evt.stop_propagation();
            CardKeyAction::Expand
        }
        Key::Character(ref c) if c == " " => {
            evt.prevent_default();
            evt.stop_propagation();
            CardKeyAction::Toggle
        }
        Key::Enter => {
            evt.prevent_default();
            evt.stop_propagation();
            document::eval(&js_focus_first_field());
            CardKeyAction::EnterEdit
        }
        Key::Escape => {
            evt.prevent_default();
            evt.stop_propagation();
            // Focus the parent section / tab.
            document::eval(JS_FOCUS_PARENT_SECTION);
            CardKeyAction::None
        }
        _ => CardKeyAction::None,
    }
}

/// Focus the active editor sub-tab so the user can navigate away via arrow
/// keys. Falls back to blurring the active element.
pub const JS_FOCUS_PARENT_SECTION: &str = "\
    (() => {\
        let tab = document.querySelector('[role=\"tab\"][aria-selected=\"true\"]');\
        if (tab) { tab.focus(); return; }\
        document.activeElement?.blur();\
    })()";
