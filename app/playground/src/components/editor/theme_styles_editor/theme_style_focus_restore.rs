//! Focus save / restore helpers for theme style entry "jump" scenarios.
//!
//! When a user edits a base-diagram-only theme style value (e.g. changes
//! an attribute value, adds an alias, or clicks "Add attribute"), the
//! entry is copied into the user's overlay. Because user overlay entries
//! are sorted above base-only entries, the card "jumps" to a new
//! position in the list. The browser loses focus, the card may be
//! collapsed at its new position, and the user has to hunt for it.
//!
//! This module provides two JavaScript snippets and a Rust helper that
//! work together to fix this:
//!
//! 1. [`JS_THEME_FOCUS_SAVE`] -- captures the `data-input-diagram-field` value
//!    of the card that contains the currently focused element, plus information
//!    about the inner focused element (tag name, index among siblings of the
//!    same type, and a `data-action` attribute if present). The snapshot is
//!    stored on `window.__themeStyleFocusRestore`.
//!
//! 2. [`JS_THEME_FOCUS_RESTORE`] -- runs inside `requestAnimationFrame` (so the
//!    DOM has been updated by Dioxus) and:
//!    - Finds the card by its `data-input-diagram-field` value.
//!    - If the card is collapsed (has a click-to-expand summary row), expands
//!      it by clicking the summary.
//!    - Scrolls the card into view.
//!    - Waits a frame for the expanded content to render, then re-focuses the
//!      inner element the user was interacting with.
//!
//! 3. [`theme_style_focus_save_restore`] -- a Rust helper that evaluates the
//!    save snippet, runs a caller-provided closure (which typically writes to
//!    the `input_diagram` signal), and then evaluates the restore snippet.

use dioxus::document;

/// JavaScript snippet that snapshots the focused element's card context.
///
/// Stores on `window.__themeStyleFocusRestore`:
///
/// - `fieldId`: the `data-input-diagram-field` value of the nearest ancestor
///   card. This is the stable identifier that survives the positional "jump".
/// - `tagName`: the lowercase tag name of the focused element (e.g. `"input"`,
///   `"select"`, `"button"`).
/// - `innerIndex`: the zero-based index of the focused element among siblings
///   of the same tag name within the card. This allows us to re-focus the
///   correct input when there are multiple inputs in a card.
/// - `dataAction`: the `data-action` attribute of the focused element, if any
///   (e.g. `"remove"`). This disambiguates buttons.
/// - `isButton`: whether the focused element is a `<button>` (including "Add"
///   buttons that live outside the card).
/// - `buttonText`: trimmed `textContent` of the button, used to find "Add"
///   buttons that don't live inside a card.
/// - `placeholder`: the `placeholder` attribute of the focused element, if any.
///   Used as a secondary disambiguator for inputs.
///
/// Call this **before** the signal write that triggers the re-render.
const JS_THEME_FOCUS_SAVE: &str = "\
(() => {\
    var FIELD_ATTR = 'data-input-diagram-field';\
    var el = document.activeElement;\
    if (!el || el === document.body) { window.__themeStyleFocusRestore = null; return; }\
    var card = el.closest('[' + FIELD_ATTR + ']');\
    var fieldId = card ? card.getAttribute(FIELD_ATTR) : null;\
    var tagName = el.tagName ? el.tagName.toLowerCase() : null;\
    var dataAction = el.getAttribute ? (el.getAttribute('data-action') || null) : null;\
    var placeholder = el.getAttribute ? (el.getAttribute('placeholder') || null) : null;\
    var isButton = tagName === 'button';\
    var buttonText = isButton ? (el.textContent || '').trim() : null;\
    var innerIndex = 0;\
    if (card && tagName) {\
        var siblings = card.querySelectorAll(tagName);\
        for (var i = 0; i < siblings.length; i++) {\
            if (siblings[i] === el) { innerIndex = i; break; }\
        }\
    }\
    window.__themeStyleFocusRestore = {\
        fieldId: fieldId,\
        tagName: tagName,\
        innerIndex: innerIndex,\
        dataAction: dataAction,\
        placeholder: placeholder,\
        isButton: isButton,\
        buttonText: buttonText\
    };\
})()";

/// JavaScript snippet that restores focus after a DOM update.
///
/// Uses a two-phase `requestAnimationFrame` approach:
///
/// 1. **Phase 1** (first rAF): find the card, detect collapsed state by
///    checking for the absence of `input`/`select` elements, expand it by
///    clicking the first child div (the summary row), and scroll it into view.
/// 2. **Phase 2** (second rAF): the expanded content has rendered, so find and
///    focus the inner element.
///
/// Call this **after** the signal write that triggers the re-render.
const JS_THEME_FOCUS_RESTORE: &str = "\
requestAnimationFrame(() => {\
    var restore = window.__themeStyleFocusRestore;\
    window.__themeStyleFocusRestore = null;\
    if (!restore) return;\
    var FIELD_ATTR = 'data-input-diagram-field';\
    var card = restore.fieldId\
        ? document.querySelector('[' + FIELD_ATTR + '=\"' + restore.fieldId + '\"]')\
        : null;\
    if (!card) return;\
    var needsExpand = false;\
    var hasInputs = card.querySelector('input, select');\
    if (!hasInputs) {\
        var firstChild = card.children[0];\
        if (firstChild) {\
            needsExpand = true;\
            firstChild.click();\
        }\
    }\
    card.scrollIntoView({ behavior: 'smooth', block: 'nearest' });\
    function tryFocusInner() {\
        if (!restore.tagName) { card.focus(); return; }\
        var candidates = card.querySelectorAll(restore.tagName);\
        if (candidates.length === 0) { card.focus(); return; }\
        var target = null;\
        if (restore.dataAction) {\
            for (var i = 0; i < candidates.length; i++) {\
                if (candidates[i].getAttribute('data-action') === restore.dataAction) {\
                    target = candidates[i]; break;\
                }\
            }\
        }\
        if (!target && restore.placeholder) {\
            for (var i = 0; i < candidates.length; i++) {\
                if (candidates[i].getAttribute('placeholder') === restore.placeholder) {\
                    target = candidates[i]; break;\
                }\
            }\
        }\
        if (!target && restore.innerIndex < candidates.length) {\
            target = candidates[restore.innerIndex];\
        }\
        if (!target) { target = candidates[0]; }\
        if (target) {\
            target.focus();\
            if (target.tagName.toLowerCase() === 'input' && target.type !== 'checkbox') {\
                try { target.select(); } catch(e) {}\
            }\
        } else {\
            card.focus();\
        }\
    }\
    if (needsExpand) {\
        requestAnimationFrame(() => { requestAnimationFrame(() => { tryFocusInner(); }); });\
    } else {\
        tryFocusInner();\
    }\
})";

/// Saves the current focus context, runs the provided closure (which
/// should perform the signal write that triggers the re-render), and
/// then schedules focus restoration for after the DOM update.
///
/// # Usage
///
/// Call this from any event handler that modifies theme style data in a
/// way that might cause a card to "jump" position (e.g. editing a base
/// value which promotes it to the user overlay).
///
/// ```rust,ignore
/// theme_style_focus_save_restore(|| {
///     let base = base_diagram.read();
///     let mut diagram = input_diagram.write();
///     if let Some(partials) = target.write_entry_mut(&base, &mut diagram, &parsed_key) {
///         partials.partials.insert(attr, String::new());
///     }
/// });
/// ```
pub(crate) fn theme_style_focus_save_restore(action: impl FnOnce()) {
    // Phase 1: snapshot the currently focused element's card context.
    document::eval(JS_THEME_FOCUS_SAVE);

    // Phase 2: perform the action that mutates the signal (triggers re-render).
    action();

    // Phase 3: schedule focus restoration after the DOM update.
    document::eval(JS_THEME_FOCUS_RESTORE);
}
