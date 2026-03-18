//! Focus save / restore helpers for undo and redo.
//!
//! When an undo or redo operation removes the currently focused element
//! from the DOM, the browser moves focus to `<body>`, which means the
//! `onkeydown` handler on the editor root no longer receives subsequent
//! keyboard shortcuts.
//!
//! This module provides two JavaScript snippets that work together:
//!
//! 1. [`JS_FOCUS_SAVE`] -- captures the `data-input-diagram-field` value of the
//!    active element's nearest field ancestor, as well as its next and previous
//!    sibling fields. The snapshot is stored on `window.__focusRestore`.
//!
//! 2. [`JS_FOCUS_RESTORE`] -- runs inside `requestAnimationFrame` (so the DOM
//!    has been updated by Dioxus) and checks whether the browser still has a
//!    meaningful focused element. If so, nothing happens. Otherwise it attempts
//!    to re-focus:
//!    - The same `[data-input-diagram-field]` element (by its stable ID value).
//!    - The next `[data-input-diagram-field]` sibling (by selector).
//!    - The previous `[data-input-diagram-field]` sibling (by selector).
//!    - The `#disposition_editor` container as a last resort.

/// JavaScript snippet that snapshots the `data-input-diagram-field` value
/// of the currently focused element and its neighbouring field siblings.
///
/// The snapshot is stored as `window.__focusRestore` with three nullable
/// CSS selectors: `self`, `next`, and `prev`.
///
/// - `self`: a selector for the nearest `[data-input-diagram-field]` ancestor
///   of the active element. Because the attribute value is the field's logical
///   ID (e.g. `"thing_0"`, `"proc_app_dev"`), this selector is stable across
///   re-renders.
/// - `next`: a selector for the next sibling element that has a
///   `data-input-diagram-field` attribute. This is used so that after an
///   undo/redo the focus lands on a meaningful editor field rather than an
///   arbitrary DOM sibling.
/// - `prev`: a selector for the previous sibling element with
///   `data-input-diagram-field`.
///
/// Call this **before** the undo/redo signal write so the active element
/// is still in the DOM.
pub const JS_FOCUS_SAVE: &str = "\
(() => {\
    var FIELD_ATTR = 'data-input-diagram-field';\
    function fieldSel(field) {\
        if (!field) return null;\
        return '[' + FIELD_ATTR + '=\"' + field.getAttribute(FIELD_ATTR) + '\"]';\
    }\
    function fieldSiblingSel(field, direction) {\
        if (!field) return null;\
        var sib = (direction === 'next') ? field.nextElementSibling : field.previousElementSibling;\
        while (sib) {\
            if (sib.hasAttribute(FIELD_ATTR)) {\
                return '[' + FIELD_ATTR + '=\"' + sib.getAttribute(FIELD_ATTR) + '\"]';\
            }\
            sib = (direction === 'next') ? sib.nextElementSibling : sib.previousElementSibling;\
        }\
        return null;\
    }\
    var el = document.activeElement;\
    var field = el ? el.closest('[' + FIELD_ATTR + ']') : null;\
    window.__focusRestore = {\
        self: fieldSel(field),\
        next: fieldSiblingSel(field, 'next'),\
        prev: fieldSiblingSel(field, 'prev')\
    };\
})()";

/// JavaScript snippet that restores focus after a DOM update.
///
/// Uses `requestAnimationFrame` to wait for Dioxus to flush the new
/// virtual DOM into the real DOM. If the browser still has a meaningful
/// focused element (i.e. not `<body>` and not the `#disposition_editor`
/// container), the re-render did not affect the focused element and no
/// action is taken. Otherwise it tries the saved selectors in order:
/// `self` -> `next` -> `prev` -> `#disposition_editor`.
///
/// Call this **after** the undo/redo signal write.
pub const JS_FOCUS_RESTORE: &str = "\
requestAnimationFrame(() => {\
    var focusRestore = window.__focusRestore;\
    window.__focusRestore = null;\
    var activeElement = document.activeElement;\
    if (activeElement && activeElement !== document.body && activeElement.id !== 'disposition_editor') return;\
    if (!focusRestore) { var dispositionEditor = document.getElementById('disposition_editor'); if (dispositionEditor) dispositionEditor.focus(); return; }\
    function tryFocus(sel) {\
        if (!sel) return false;\
        try { var el = document.querySelector(sel); if (el) { el.focus(); return document.activeElement === el; } } catch(e) {}\
        return false;\
    }\
    if (tryFocus(focusRestore.self)) return;\
    if (tryFocus(focusRestore.next)) return;\
    if (tryFocus(focusRestore.prev)) return;\
    var editor = document.getElementById('disposition_editor');\
    if (editor) editor.focus();\
})";
