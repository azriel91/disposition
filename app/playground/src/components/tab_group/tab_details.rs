use dioxus::prelude::Element;

/// Details for a single tab within a [`TabGroup`].
///
/// [`TabGroup`]: super::TabGroup
#[derive(Clone, PartialEq)]
pub struct TabDetails {
    /// Text displayed on the tab label.
    pub label: String,
    /// Content rendered inside the tab panel when the tab is active.
    pub content: Element,
}
