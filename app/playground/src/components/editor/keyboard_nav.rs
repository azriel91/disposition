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
//! - **Ctrl+Up / Ctrl+Down**: jump to the first / last sibling entry.
//! - **Alt+Up / Alt+Down**: reorder the entry up or down in the list.
//! - **Left**: collapse the entry (if collapsible).
//! - **Right**: expand the entry (if collapsible).
//! - **Space**: toggle collapsed state (if collapsible).
//! - **Enter**: expand (if collapsed) and focus the first interactive element
//!   inside the entry.
//! - **Ctrl+Shift+K**: remove the entry.
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
//!
//! ## Focus After Remove
//!
//! [`js_focus_after_field_remove`] provides a unified strategy for restoring
//! focus when an editor field (`IdValueRow`, `*Card`, `StyleAliasesSection`)
//! is removed via **Ctrl+Shift+K**. It walks up from the active element to
//! the nearest `[data-input-diagram-field]` ancestor, then tries, in order:
//!
//! 1. The next sibling with `[data-input-diagram-field]`.
//! 2. The previous sibling with `[data-input-diagram-field]`.
//! 3. The closest focusable ancestor.
//!
//! ## Focus Save / Restore for Undo/Redo
//!
//! The `JS_FOCUS_SAVE` constant in `focus_restore` captures the
//! `data-input-diagram-field` values of the active element's next and
//! previous sibling fields, storing them on `window.__focusRestore` for
//! `JS_FOCUS_RESTORE` to use after a DOM update.

use dioxus::{
    document,
    prelude::{Key, ModifiersInteraction},
};

use crate::components::editor::common::DATA_INPUT_DIAGRAM_FIELD;

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

/// Build a JS snippet that focuses the **first** sibling element matching
/// `[data_attr]` within the same parent container.
///
/// Used by Ctrl+Up to jump to the top of a reorderable list.
pub fn js_focus_first_entry(data_attr: &str) -> String {
    format!(
        "(() => {{\
            let el = document.activeElement;\
            if (!el) return;\
            let card = el.closest('[{data_attr}]') || el;\
            let container = card.parentElement;\
            if (!container) return;\
            let first = container.querySelector('[{data_attr}]');\
            if (first && first !== card) first.focus();\
        }})()"
    )
}

/// Build a JS snippet that focuses the **last** sibling element matching
/// `[data_attr]` within the same parent container.
///
/// Used by Ctrl+Down to jump to the bottom of a reorderable list.
pub fn js_focus_last_entry(data_attr: &str) -> String {
    format!(
        "(() => {{\
            let el = document.activeElement;\
            if (!el) return;\
            let card = el.closest('[{data_attr}]') || el;\
            let container = card.parentElement;\
            if (!container) return;\
            let all = container.querySelectorAll('[{data_attr}]');\
            let last = all[all.length - 1];\
            if (last && last !== card) last.focus();\
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

// === Focus after remove === //

/// Build a JS snippet that schedules focus on the next (or previous, or
/// ancestor) `[data-input-diagram-field]` sibling after the current field
/// is removed.
///
/// The snippet stores the target selector on `window.__focusAfterRemove`
/// **before** the DOM mutation. A `requestAnimationFrame` callback then
/// focuses the element after Dioxus has flushed the update.
///
/// Resolution order:
///
/// 1. Next sibling element with `[data-input-diagram-field]`.
/// 2. Previous sibling element with `[data-input-diagram-field]`.
/// 3. Closest focusable ancestor (e.g. the editor tab).
pub fn js_focus_after_field_remove() -> String {
    let attr = DATA_INPUT_DIAGRAM_FIELD;
    format!(
        "(() => {{\
            var el = document.activeElement;\
            if (!el) return;\
            var field = el.closest('[{attr}]') || el;\
            var sel = null;\
            var next = field.nextElementSibling;\
            while (next) {{\
                if (next.hasAttribute('{attr}')) {{\
                    sel = '[{attr}=\"' + next.getAttribute('{attr}') + '\"]';\
                    break;\
                }}\
                next = next.nextElementSibling;\
            }}\
            if (!sel) {{\
                var prev = field.previousElementSibling;\
                while (prev) {{\
                    if (prev.hasAttribute('{attr}')) {{\
                        sel = '[{attr}=\"' + prev.getAttribute('{attr}') + '\"]';\
                        break;\
                    }}\
                    prev = prev.previousElementSibling;\
                }}\
            }}\
            if (!sel) {{\
                var parent = field.parentElement;\
                while (parent) {{\
                    if (parent.tabIndex >= 0) {{\
                        sel = parent.id ? '#' + CSS.escape(parent.id) : null;\
                        break;\
                    }}\
                    parent = parent.parentElement;\
                }}\
            }}\
            window.__focusAfterRemove = sel;\
            requestAnimationFrame(() => {{\
                var target = window.__focusAfterRemove;\
                window.__focusAfterRemove = null;\
                if (target) {{\
                    try {{ var e = document.querySelector(target); if (e) {{ e.focus(); return; }} }} catch(ex) {{}}\
                }}\
                var editor = document.getElementById('disposition_editor');\
                if (editor) editor.focus();\
            }});\
        }})()"
    )
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
    /// The user pressed **Alt+Up** -- move the entry up in the list.
    MoveUp,
    /// The user pressed **Alt+Down** -- move the entry down in the list.
    MoveDown,
    /// The user pressed **Alt+Shift+Up** -- insert a new entry above.
    AddAbove,
    /// The user pressed **Alt+Shift+Down** -- insert a new entry below.
    AddBelow,
    /// The user pressed **Ctrl+Shift+K** -- remove the entry.
    Remove,
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
    let alt = evt.modifiers().alt();
    let ctrl = evt.modifiers().ctrl();
    let shift = evt.modifiers().shift();

    match evt.key() {
        // === Ctrl+Shift+K: remove entry === //
        Key::Character(ref c) if ctrl && shift && c.eq_ignore_ascii_case("k") => {
            evt.prevent_default();
            evt.stop_propagation();
            CardKeyAction::Remove
        }
        Key::ArrowUp if alt && shift => {
            evt.prevent_default();
            evt.stop_propagation();
            CardKeyAction::AddAbove
        }
        Key::ArrowDown if alt && shift => {
            evt.prevent_default();
            evt.stop_propagation();
            CardKeyAction::AddBelow
        }
        Key::ArrowUp if alt => {
            evt.prevent_default();
            evt.stop_propagation();
            CardKeyAction::MoveUp
        }
        Key::ArrowDown if alt => {
            evt.prevent_default();
            evt.stop_propagation();
            CardKeyAction::MoveDown
        }
        Key::ArrowUp if ctrl => {
            evt.prevent_default();
            evt.stop_propagation();
            document::eval(&js_focus_first_entry(data_attr));
            CardKeyAction::None
        }
        Key::ArrowDown if ctrl => {
            evt.prevent_default();
            evt.stop_propagation();
            document::eval(&js_focus_last_entry(data_attr));
            CardKeyAction::None
        }
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
