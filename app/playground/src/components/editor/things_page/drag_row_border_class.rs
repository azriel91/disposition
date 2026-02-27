//! Drag-and-drop border class helper.
//!
//! Computes the Tailwind border-colour classes for a draggable row based on
//! whether it is currently the drop target.

use dioxus::signals::{ReadableExt, Signal};

/// Returns Tailwind border-color classes for the drop-target indicator.
///
/// Always returns **both** `border-t-*` and `border-b-*` colour classes so
/// that there is never a cascade conflict with a competing colour class on
/// the same element (Tailwind v4 orders utilities by property, not by the
/// order they appear in the `class` attribute).
///
/// - When this row is the drop target and the drag source is above, the bottom
///   border turns blue (`border-b-blue-400`) and the top stays transparent.
/// - When the drag source is below, the top border turns blue
///   (`border-t-blue-400`) and the bottom stays transparent.
/// - Otherwise both borders are transparent.
pub fn drag_row_border_class(
    drag_index: Signal<Option<usize>>,
    drop_target: Signal<Option<usize>>,
    index: usize,
) -> &'static str {
    let drag_src = *drag_index.read();
    let is_target = drop_target.read().map_or(false, |i| i == index);

    if is_target {
        if let Some(from) = drag_src {
            if from != index {
                if from < index {
                    return "border-t-transparent border-b-blue-400";
                } else {
                    return "border-t-blue-400 border-b-transparent";
                }
            }
        }
    }
    "border-t-transparent border-b-transparent"
}
