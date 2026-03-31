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
//!    of the card that contains the currently focused element, **plus** the
//!    field IDs of all ancestor `[data-input-diagram-field]` elements. This
//!    ancestor chain allows the restore logic to scope its search to the
//!    correct section when multiple sections share the same inner card key
//!    (e.g. multiple `TypesStylesSection` instances each containing a
//!    `"node_defaults"` card). The snapshot is stored on
//!    `window.__themeStyleFocusRestore`.
//!
//! 2. [`JS_THEME_FOCUS_RESTORE`] -- runs inside `requestAnimationFrame` (so the
//!    DOM has been updated by Dioxus) and:
//!    - Walks the ancestor chain from outermost to innermost to find the
//!      correct scoping container, then locates the card within it.
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
/// - `ancestorFieldIds`: an array of `data-input-diagram-field` values from the
///   card's parent elements, ordered from nearest ancestor outward. Used to
///   scope the restore search when multiple sections share the same inner card
///   key (e.g. multiple `TypesStylesSection` components each containing a
///   `"node_defaults"` card).
/// - `isButton`: whether the focused element is a `<button>`.
/// - `precedingFieldIndex`: when `isButton` is true, the zero-based index of
///   the closest `input`/`select`/`textarea` that appears before the button in
///   DOM order within the card. On restore, the element at index
///   `precedingFieldIndex + 1` is focused -- i.e. the newly inserted field.
///   `-1` if no preceding field exists (restore will target index `0`).
/// - `tagName`: the lowercase tag name of the focused element (e.g. `"input"`,
///   `"select"`, `"button"`).
/// - `innerIndex`: the zero-based index of the focused element among siblings
///   of the same tag name within the card. This allows us to re-focus the
///   correct input when there are multiple inputs in a card.
/// - `dataAction`: the `data-action` attribute of the focused element, if any
///   (e.g. `"remove"`). This disambiguates buttons.
/// - `placeholder`: the `placeholder` attribute of the focused element, if any.
///   Used as a secondary disambiguator for inputs.
///
/// Call this **before** the signal write that triggers the re-render.
const JS_THEME_FOCUS_SAVE: &str = "\
(() => {\
    var FIELD_ATTR = 'data-input-diagram-field';\
    var FIELD_SEL = 'input, select, textarea';\
    var el = document.activeElement;\
    if (!el || el === document.body) { window.__themeStyleFocusRestore = null; return; }\
    var card = el.closest('[' + FIELD_ATTR + ']');\
    var fieldId = card ? card.getAttribute(FIELD_ATTR) : null;\
    var ancestorFieldIds = [];\
    if (card) {\
        var ancestor = card.parentElement;\
        while (ancestor) {\
            if (ancestor.hasAttribute && ancestor.hasAttribute(FIELD_ATTR)) {\
                ancestorFieldIds.push(ancestor.getAttribute(FIELD_ATTR));\
            }\
            ancestor = ancestor.parentElement;\
        }\
    }\
    var tagName = el.tagName ? el.tagName.toLowerCase() : null;\
    var dataAction = el.getAttribute ? (el.getAttribute('data-action') || null) : null;\
    var placeholder = el.getAttribute ? (el.getAttribute('placeholder') || null) : null;\
    var isButton = tagName === 'button';\
    var precedingFieldIndex = -1;\
    if (isButton && card) {\
        var allFields = card.querySelectorAll(FIELD_SEL);\
        for (var fi = allFields.length - 1; fi >= 0; fi--) {\
            var cmp = el.compareDocumentPosition(allFields[fi]);\
            if (cmp & Node.DOCUMENT_POSITION_PRECEDING) {\
                precedingFieldIndex = fi;\
                break;\
            }\
        }\
    }\
    var innerIndex = 0;\
    if (card && tagName) {\
        var siblings = card.querySelectorAll(tagName);\
        for (var i = 0; i < siblings.length; i++) {\
            if (siblings[i] === el) { innerIndex = i; break; }\
        }\
    }\
    window.__themeStyleFocusRestore = {\
        fieldId: fieldId,\
        ancestorFieldIds: ancestorFieldIds,\
        isButton: isButton,\
        precedingFieldIndex: precedingFieldIndex,\
        tagName: tagName,\
        innerIndex: innerIndex,\
        dataAction: dataAction,\
        placeholder: placeholder\
    };\
})()";

/// JavaScript snippet that restores focus after a DOM update.
///
/// Uses a two-phase `requestAnimationFrame` approach:
///
/// 1. **Phase 1** (first rAF): find the card by walking the ancestor chain from
///    outermost to innermost, detect collapsed state by checking for the
///    absence of `input`/`select` elements, expand it by clicking the first
///    child div (the summary row), and scroll it into view.
/// 2. **Phase 2** (second rAF): the expanded content has rendered, so find and
///    focus the inner element.
///
/// The ancestor chain (`ancestorFieldIds`) is used to scope the search so
/// that when multiple sections share the same inner card key (e.g. two
/// `TypesStylesSection` components both containing a `"node_defaults"` card),
/// the correct section's card is found.
///
/// For button-triggered actions (e.g. "+ Add attribute"), the restore logic
/// focuses the `input`/`select`/`textarea` at index `precedingFieldIndex + 1`
/// within the card -- i.e. the first field in the newly inserted row.
///
/// Call this **after** the signal write that triggers the re-render.
const JS_THEME_FOCUS_RESTORE: &str = "\
requestAnimationFrame(() => {\
    var restore = window.__themeStyleFocusRestore;\
    window.__themeStyleFocusRestore = null;\
    if (!restore || !restore.fieldId) return;\
    var FIELD_ATTR = 'data-input-diagram-field';\
    var FIELD_SEL = 'input, select, textarea';\
    var sel = '[' + FIELD_ATTR + '=\"' + restore.fieldId + '\"]';\
    var card = null;\
    var ancestors = restore.ancestorFieldIds || [];\
    if (ancestors.length > 0) {\
        var scope = null;\
        for (var i = ancestors.length - 1; i >= 0; i--) {\
            var ancSel = '[' + FIELD_ATTR + '=\"' + ancestors[i] + '\"]';\
            var searchIn = scope || document;\
            scope = searchIn.querySelector(ancSel);\
            if (!scope) break;\
        }\
        if (scope) {\
            card = scope.querySelector(sel);\
        }\
    }\
    if (!card) {\
        card = document.querySelector(sel);\
    }\
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
        if (restore.isButton) {\
            var allFields = card.querySelectorAll(FIELD_SEL);\
            var targetIdx = restore.precedingFieldIndex + 1;\
            if (targetIdx >= 0 && targetIdx < allFields.length) {\
                var t = allFields[targetIdx];\
                t.focus();\
                if (t.tagName.toLowerCase() === 'input' && t.type !== 'checkbox') {\
                    try { t.select(); } catch(e) {}\
                }\
                return;\
            }\
            if (allFields.length > 0) {\
                allFields[allFields.length - 1].focus();\
                return;\
            }\
            card.focus();\
            return;\
        }\
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
