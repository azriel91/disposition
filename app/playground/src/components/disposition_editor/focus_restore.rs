//! Focus save / restore helpers for undo and redo.
//!
//! When an undo or redo operation removes the currently focused element
//! from the DOM, the browser moves focus to `<body>`, which means the
//! `onkeydown` handler on the editor root no longer receives subsequent
//! keyboard shortcuts.
//!
//! This module provides two JavaScript snippets that work together:
//!
//! 1. [`JS_FOCUS_SAVE`] -- captures the active element's identity (a CSS
//!    selector built from `id`, `data-*` attributes, or sibling index) together
//!    with the identity of its next and previous siblings. The snapshot is
//!    stored on `window.__focusRestore`.
//!
//! 2. [`JS_FOCUS_RESTORE`] -- runs inside `requestAnimationFrame` (so the DOM
//!    has been updated by Dioxus) and attempts to re-focus:
//!    - The original element (by selector).
//!    - The original element's next sibling (by selector).
//!    - The original element's previous sibling (by selector).
//!    - The `#disposition_editor` container as a last resort.

/// JavaScript snippet that snapshots the currently focused element and
/// its immediate siblings.
///
/// The snapshot is stored as `window.__focusRestore` with three nullable
/// CSS selectors: `self`, `next`, and `prev`.
///
/// Call this **before** the undo/redo signal write so the active element
/// is still in the DOM.
pub const JS_FOCUS_SAVE: &str = "\
(() => {\
    function selectorFor(el) {\
        if (!el || el === document.body) return null;\
        if (el.id) return '#' + CSS.escape(el.id);\
        var ds = el.dataset;\
        for (var k in ds) {\
            if (Object.prototype.hasOwnProperty.call(ds, k)) {\
                var attr = 'data-' + k.replace(/([A-Z])/g, '-$1').toLowerCase();\
                return '[' + attr + '=' + JSON.stringify(ds[k]) + ']';\
            }\
        }\
        var parent = el.parentElement;\
        if (!parent) return null;\
        var children = parent.children;\
        for (var i = 0; i < children.length; i++) {\
            if (children[i] === el) {\
                var ps = selectorFor(parent);\
                if (ps) return ps + ' > :nth-child(' + (i + 1) + ')';\
                break;\
            }\
        }\
        return null;\
    }\
    var el = document.activeElement;\
    window.__focusRestore = {\
        self: selectorFor(el),\
        next: el ? selectorFor(el.nextElementSibling) : null,\
        prev: el ? selectorFor(el.previousElementSibling) : null\
    };\
})()";

/// JavaScript snippet that restores focus after a DOM update.
///
/// Uses `requestAnimationFrame` to wait for Dioxus to flush the new
/// virtual DOM into the real DOM, then tries the saved selectors in
/// order: `self` -> `next` -> `prev` -> `#disposition_editor`.
///
/// Call this **after** the undo/redo signal write.
pub const JS_FOCUS_RESTORE: &str = "\
requestAnimationFrame(() => {\
    var r = window.__focusRestore;\
    if (!r) { var fb = document.getElementById('disposition_editor'); if (fb) fb.focus(); return; }\
    window.__focusRestore = null;\
    function tryFocus(sel) {\
        if (!sel) return false;\
        try { var el = document.querySelector(sel); if (el) { el.focus(); return document.activeElement === el; } } catch(e) {}\
        return false;\
    }\
    if (tryFocus(r.self)) return;\
    if (tryFocus(r.next)) return;\
    if (tryFocus(r.prev)) return;\
    var editor = document.getElementById('disposition_editor');\
    if (editor) editor.focus();\
})";
