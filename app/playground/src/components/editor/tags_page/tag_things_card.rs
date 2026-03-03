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

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::InputDiagram;

use crate::components::editor::{
    common::{CardComponent, FieldNav, RenameRefocus, ADD_BTN, REMOVE_BTN, ROW_CLASS_SIMPLE},
    datalists::list_ids,
    reorderable::{drag_border_class, DragHandle},
};

use super::{
    tags_page_ops::TagsPageOps, COLLAPSED_HEADER_CLASS, DATA_ATTR, FIELD_INPUT_CLASS,
    TAG_THINGS_CARD_CLASS,
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
    let thing_suffix = if thing_count != 1 { "s" } else { "" };

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
                    TagsPageOps::tag_things_entry_move(input_diagram, index, index - 1);
                    focus_index.set(Some(index - 1));
                },
                move || {
                    TagsPageOps::tag_things_entry_move(input_diagram, index, index + 1);
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
                    TagsPageOps::tag_things_entry_move(input_diagram, from, index);
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
                div {
                    class: COLLAPSED_HEADER_CLASS,
                    onclick: move |_| collapsed.set(false),

                    DragHandle {}

                    // Expand chevron
                    span {
                        class: "text-gray-500 text-xs",
                        ">"
                    }

                    span {
                        class: "text-sm font-mono text-blue-400",
                        "{tag_id}"
                    }

                    span {
                        class: "text-xs text-gray-500",
                        "({thing_count} thing{thing_suffix})"
                    }
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
                div {
                    class: ROW_CLASS_SIMPLE,

                    label {
                        class: "text-xs text-gray-500 w-12",
                        "Tag"
                    }

                    input {
                        class: FIELD_INPUT_CLASS,
                        style: "max-width:14rem",
                        tabindex: "-1",
                        list: list_ids::TAG_IDS,
                        placeholder: "tag_id",
                        value: "{tag_id}",
                        onchange: {
                            let tag_id_old = tag_id.clone();
                            let current_things = things.clone();
                            move |evt: dioxus::events::FormEvent| {
                                let id_new = evt.value();
                                let target = *rename_target.read();
                                TagsPageOps::tag_things_entry_rename(
                                    input_diagram,
                                    &tag_id_old,
                                    &id_new,
                                    &current_things,
                                );
                                rename_refocus.set(Some(RenameRefocus {
                                    new_id: id_new,
                                    target,
                                }));
                            }
                        },
                        onkeydown: FieldNav::id_onkeydown(DATA_ATTR, rename_target)
                    }

                    button {
                        class: REMOVE_BTN,
                        tabindex: "-1",
                        "data-action": "remove",
                        onclick: {
                            let tag_id = tag_id.clone();
                            move |_| {
                                TagsPageOps::tag_things_entry_remove(input_diagram, &tag_id);
                            }
                        },
                        onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                        "x Remove"
                    }
                }

                // === Thing list === //
                div {
                    class: "flex flex-col gap-1 pl-4",

                    for (idx, thing_id) in things.iter().enumerate() {
                        {
                            let thing_id = thing_id.clone();
                            let tag_id = tag_id.clone();
                            rsx! {
                                div {
                                    key: "{tag_id}_{idx}",
                                    class: ROW_CLASS_SIMPLE,

                                    span {
                                        class: "text-xs text-gray-500 w-6 text-right",
                                        "{idx}."
                                    }

                                    input {
                                        class: FIELD_INPUT_CLASS,
                                        style: "max-width:14rem",
                                        tabindex: "-1",
                                        list: list_ids::THING_IDS,
                                        placeholder: "thing_id",
                                        value: "{thing_id}",
                                        onchange: {
                                            let tag_id = tag_id.clone();
                                            move |evt: dioxus::events::FormEvent| {
                                                TagsPageOps::tag_things_thing_update(
                                                    input_diagram,
                                                    &tag_id,
                                                    idx,
                                                    &evt.value(),
                                                );
                                            }
                                        },
                                        onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                                    }

                                    button {
                                        class: REMOVE_BTN,
                                        tabindex: "-1",
                                        "data-action": "remove",
                                        onclick: {
                                            let tag_id = tag_id.clone();
                                            move |_| {
                                                TagsPageOps::tag_things_thing_remove(
                                                    input_diagram,
                                                    &tag_id,
                                                    idx,
                                                );
                                            }
                                        },
                                        onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                                        "\u{2715}"
                                    }
                                }
                            }
                        }
                    }

                    button {
                        class: ADD_BTN,
                        tabindex: -1,
                        onclick: {
                            let tag_id = tag_id.clone();
                            move |_| {
                                TagsPageOps::tag_things_thing_add(input_diagram, &tag_id);
                            }
                        },
                        onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                        "+ Add thing"
                    }
                }
            }
        }
    }
}
