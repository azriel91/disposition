//! Drag handle component.
//!
//! Provides a purely-visual grip indicator for draggable rows.

use dioxus::prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element};

use crate::components::editor::common::DRAG_HANDLE;

/// A grip-dots drag handle that visually indicates a row is draggable.
///
/// The actual drag-and-drop behaviour is handled by the parent row's
/// `draggable` / `ondragstart` / `ondragover` / `ondrop` / `ondragend`
/// attributes; this component is purely visual.
#[component]
pub fn DragHandle() -> Element {
    rsx! {
        span {
            class: DRAG_HANDLE,
            title: "Drag to reorder",
            "⠿"
        }
    }
}
