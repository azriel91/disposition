//! Collapsible card component for a single tag's associated things.
//!
//! Displays the tag ID, a remove button, and a list of thing IDs
//! that can be individually edited, removed, or added to.
//!
//! Supports keyboard shortcuts:
//!
//! - **ArrowUp / ArrowDown**: navigate between sibling cards.
//! - **Alt+Up / Alt+Down**: move the card up or down in the list.
//! - **ArrowRight**: expand the card (when collapsed).
//! - **ArrowLeft**: collapse the card (when expanded).
//! - **Space**: toggle expand/collapse.
//! - **Enter**: expand + focus the first input inside the card.
//! - **Escape**: focus the parent section / tab.
//! - **Tab / Shift+Tab** (inside a field): cycle through focusable fields
//!   within the card. Wraps from last to first / first to last.
//! - **Esc** (inside a field): return focus to the card wrapper.

mod tag_things_card_field_id;
mod tag_things_card_field_things;
mod tag_things_card_field_things_row;
mod tag_things_card_summary;

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::TagsPageOps;

use crate::components::editor::{
    common::{CardComponent, RenameRefocus},
    reorderable::{drag_border_class, DragHandle},
    tags_page::{DATA_ATTR, TAG_THINGS_CARD_CLASS},
};

use self::{
    tag_things_card_field_id::TagThingsCardFieldId,
    tag_things_card_field_things::TagThingsCardFieldThings,
    tag_things_card_field_things_row::TagThingsCardFieldThingsRow,
    tag_things_card_summary::TagThingsCardSummary,
};

/// A collapsible card for editing a single tag's associated things.
///
/// When collapsed, shows the tag ID and number of things.
/// When expanded, shows the tag ID input, a remove button, and the list
/// of thing IDs with individual edit/remove controls.
#[component]
pub(crate) fn TagThingsCard(
    input_diagram: Signal<InputDiagram<'static>>,
    tag_id: String,
    things: Vec<String>,
    index: usize,
    entry_count: usize,
    drag_index: Signal<Option<usize>>,
    drop_target: Signal<Option<usize>>,
    mut focus_index: Signal<Option<usize>>,
    mut rename_refocus: Signal<Option<RenameRefocus>>,
) -> Element {
    let card_state =
        CardComponent::state_init_with_rename(index, entry_count, rename_refocus, &tag_id);
    let mut collapsed = card_state.collapsed;
    let rename_target = card_state.rename_target;
    let border_class = drag_border_class(drag_index, drop_target, index);

    let thing_count = things.len();

    rsx! {
        div {
            class: "{TAG_THINGS_CARD_CLASS} {border_class}",
            tabindex: "0",
            draggable: "true",
            "data-tag-things-card": "true",

            // === Card identity for post-rename focus === //
            "data-tag-things-card-id": "{tag_id}",

            // === Card-level keyboard shortcuts === //
            onkeydown: CardComponent::card_onkeydown(
                DATA_ATTR,
                card_state,
                move || {
                    TagsPageOps::tag_things_entry_move(&mut input_diagram.write(), index, index - 1);
                    focus_index.set(Some(index - 1));
                },
                move || {
                    TagsPageOps::tag_things_entry_move(&mut input_diagram.write(), index, index + 1);
                    focus_index.set(Some(index + 1));
                },
            ),

            // === Drag-and-drop === //
            ondragstart: move |_| {
                drag_index.set(Some(index));
            },
            ondragover: move |evt| {
                evt.prevent_default();
                drop_target.set(Some(index));
            },
            ondrop: move |evt| {
                evt.prevent_default();
                if let Some(from) = *drag_index.read()
                    && from != index
                {
                    TagsPageOps::tag_things_entry_move(&mut input_diagram.write(), from, index);
                }
                drag_index.set(None);
                drop_target.set(None);
            },
            ondragend: move |_| {
                drag_index.set(None);
                drop_target.set(None);
            },

            if *collapsed.read() {
                // === Collapsed summary === //
                TagThingsCardSummary {
                    tag_id: tag_id.clone(),
                    thing_count,
                    collapsed,
                }
            } else {
                // === Expanded content === //

                // Collapse toggle + drag handle
                div {
                    class: "flex flex-row items-center gap-1 cursor-pointer select-none mb-1",
                    onclick: move |_| collapsed.set(true),

                    DragHandle {}

                    span {
                        class: "text-gray-500 text-xs rotate-90 inline-block",
                        ">"
                    }
                    span {
                        class: "text-xs text-gray-500",
                        "Collapse"
                    }
                }

                // === Header: TagId + Remove === //
                TagThingsCardFieldId {
                    input_diagram,
                    tag_id: tag_id.clone(),
                    things: things.clone(),
                    rename_target,
                    rename_refocus,
                }

                // === Thing list === //
                TagThingsCardFieldThings {
                    input_diagram,
                    tag_id: tag_id.clone(),
                    things,
                }
            }
        }
    }
}
