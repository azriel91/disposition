//! Common helper functions and shared CSS constants for editor page modules.
//!
//! ID parsers, theme-style rename helpers, and the shared
//! rename-across-diagram helper are now provided by `disposition_input_rt`.
//! This module re-exports them so that existing callers within the playground
//! continue to compile without path changes.
//!
//! Also exports [`RenameRefocus`], which carries the context needed to restore
//! keyboard focus after an ID rename causes the focused element to be
//! destroyed and recreated with a new key.

pub(crate) use self::{
    card_component::CardComponent, field_nav::FieldNav, row_component::RowComponent,
};

mod card_component;
mod field_nav;
mod row_component;

// === Re-exports from disposition_input_rt === //

pub use disposition_input_rt::id_parse::{
    parse_entity_type_id, parse_id_or_defaults, parse_tag_id_or_defaults,
};

// === Post-rename focus type === //

/// Which sub-element should receive focus after a rename-induced re-creation.
///
/// When a user renames an ID the DOM element is destroyed and recreated under a
/// new key. The triggering key determines where focus should land in the new
/// element.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenameRefocusTarget {
    /// Enter or blur triggered the rename -- re-focus the ID input.
    IdInput,
    /// Tab (forward) triggered the rename -- focus the next field after the ID
    /// input.
    NextField,
    /// Shift+Tab or Esc triggered the rename -- focus the nearest focusable
    /// ancestor (the parent row for `IdValueRow`, the card wrapper for cards).
    FocusParent,
}

/// Carries the context needed after an ID rename to restore keyboard focus.
///
/// When a user renames an ID (e.g. in an `IdValueRow` or a card component),
/// the DOM element is destroyed and recreated under a new key. A stable
/// ancestor component uses this value -- received via a shared signal -- to
/// re-focus the correct field in the new element after the DOM update.
///
/// # Fields
///
/// * `new_id`: the ID string the entry was renamed to, e.g. `"thing_1"`.
/// * `target`: which sub-element to focus after the rename.
#[derive(Clone, PartialEq)]
pub struct RenameRefocus {
    /// The new ID string after the rename, e.g. `"thing_1"`.
    pub new_id: String,
    /// Which sub-element to focus after the rename.
    pub target: RenameRefocusTarget,
}

// === Shared CSS constants === //

/// CSS classes shared by all section headings inside editor pages.
pub const SECTION_HEADING: &str = "text-sm font-bold text-gray-300 mt-4 mb-1";

/// CSS classes for the outer wrapper of a key-value row.
pub const ROW_CLASS: &str = "\
    flex flex-row gap-2 items-center \
    pt-1 \
    pb-1 \
    border-t-1 \
    border-b-1 \
    has-[:active]:opacity-40\
";

/// Row-level flex layout (no border/drag styling).
pub const ROW_CLASS_SIMPLE: &str = "flex flex-row gap-2 items-center";

/// CSS classes for text inputs.
pub const INPUT_CLASS: &str = "\
    flex-1 \
    rounded \
    border \
    border-gray-600 \
    bg-gray-800 \
    text-gray-200 \
    px-2 py-1 \
    text-sm \
    font-mono \
    focus:border-blue-400 \
    focus:outline-none\
";

/// CSS classes for ID inputs (with validation colouring).
pub const ID_INPUT_CLASS: &str = "\
    flex-1 \
    rounded \
    border \
    border-gray-600 \
    bg-gray-800 \
    text-gray-200 \
    px-2 py-1 \
    text-sm \
    font-mono \
    focus:border-blue-400 \
    focus:outline-none \
    invalid:bg-red-950 \
    invalid:border-red-400\
";

/// CSS classes for a select / dropdown.
pub const SELECT_CLASS: &str = "\
    rounded \
    border \
    border-gray-600 \
    bg-gray-800 \
    text-gray-200 \
    px-2 py-1 \
    text-sm \
    focus:border-blue-400 \
    focus:outline-none\
";

/// CSS classes for the small "remove" button.
pub const REMOVE_BTN: &str = "\
    bg-transparent \
    border-none \
    cursor-pointer \
    outline-none \
    rounded \
    p-0 \
    px-1 \
    text-xs \
    text-red-400 \
    hover:text-red-300 \
    focus:border \
    focus:border-solid \
    focus:border-blue-400\
";

/// CSS classes for the "add" button.
pub const ADD_BTN: &str = "\
    mt-1 \
    text-left \
    text-sm \
    text-blue-400 \
    hover:text-blue-300 \
    cursor-pointer \
    select-none\
";

/// CSS classes for a card-like container.
pub const CARD_CLASS: &str = "\
    rounded-lg \
    border \
    border-gray-700 \
    bg-gray-900 \
    p-3 \
    mb-2 \
    flex \
    flex-col \
    gap-2\
";

/// CSS classes for a nested card (e.g. steps within a process).
pub const INNER_CARD_CLASS: &str = "\
    rounded \
    border \
    border-gray-700 \
    bg-gray-850 \
    p-2 \
    flex \
    flex-col \
    gap-1\
";

/// CSS classes for textareas (theme / YAML editors).
pub const TEXTAREA_CLASS: &str = "\
    w-full \
    min-h-24 \
    rounded \
    border \
    border-gray-600 \
    bg-gray-800 \
    text-gray-200 \
    p-2 \
    font-mono \
    text-sm \
    focus:border-blue-400 \
    focus:outline-none\
";

/// CSS classes for the drag handle grip -- braille dots (`⠿`).
pub const DRAG_HANDLE: &str = "\
    text-gray-600 \
    hover:text-gray-400 \
    cursor-grab \
    active:cursor-grabbing \
    select-none \
    leading-none \
    text-sm \
    px-0.5 \
    flex \
    items-center\
";

/// Helper label classes.
pub const LABEL_CLASS: &str = "text-xs text-gray-500 mb-1";

/// The `data-*` attribute placed on every top-level editor field element
/// (`IdValueRow`, `ProcessCard`, `EdgeGroupCard`, `TagThingsCard`,
/// `CssClassPartialsCard`, `StyleAliasesSection`).
///
/// Used by the focus-after-remove and undo/redo focus-save/restore logic
/// to locate sibling fields on the current page regardless of their
/// concrete type.
///
/// The attribute value is the field's ID string, e.g. `"thing_0"`,
/// `"proc_app_dev"`, `"shade_light"`.
pub const DATA_INPUT_DIAGRAM_FIELD: &str = "data-input-diagram-field";
