//! Things list field for a [`TagThingsCard`].
//!
//! Extracted from [`TagThingsCard`] to keep the parent component concise.
//!
//! [`TagThingsCard`]: super::TagThingsCard

use dioxus::{
    hooks::use_signal,
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Signal, WritableExt},
};
use disposition::input_model::InputDiagram;
use disposition_input_rt::TagsPageOps;

use crate::components::editor::{
    common::{FieldNav, ADD_BTN},
    reorderable::ReorderableContainer,
    tags_page::{tag_things_card::TagThingsCardFieldThingsRow, DATA_ATTR},
};

/// Things list field inside a tag-things card.
///
/// Displays a [`ReorderableContainer`] of [`TagThingsCardFieldThingsRow`]
/// entries with full keyboard navigation and drag-and-drop reordering,
/// and an "+ Add thing" button.
#[component]
pub(crate) fn TagThingsCardFieldThings(
    input_diagram: Signal<InputDiagram<'static>>,
    tag_id: String,
    things: Vec<String>,
) -> Element {
    let thing_count = things.len();
    let thing_focus_idx: Signal<Option<usize>> = use_signal(|| None);
    let thing_drag_idx: Signal<Option<usize>> = use_signal(|| None);
    let thing_drop_target: Signal<Option<usize>> = use_signal(|| None);

    rsx! {
        div {
            class: "flex flex-col gap-1 pl-4",

            ReorderableContainer {
                data_attr: "data-tag-thing-row".to_owned(),
                section_id: format!("tag_things_{tag_id}"),
                focus_index: thing_focus_idx,
                focus_inner_selector: Some("input".to_owned()),

                for (idx, thing_id) in things.iter().enumerate() {
                    {
                        let thing_id = thing_id.clone();
                        let tag_id = tag_id.clone();
                        let tag_id_move = tag_id.clone();
                        let tag_id_add = tag_id.clone();
                        let tag_id_remove = tag_id.clone();
                        rsx! {
                            TagThingsCardFieldThingsRow {
                                key: "{tag_id}_{idx}",
                                input_diagram,
                                tag_id,
                                thing_id,
                                index: idx,
                                thing_count,
                                thing_focus_idx,
                                drag_index: thing_drag_idx,
                                drop_target: thing_drop_target,
                                on_move: move |(from, to): (usize, usize)| {
                                    TagsPageOps::tag_things_thing_move(
                                        &mut input_diagram.write(),
                                        &tag_id_move,
                                        from,
                                        to,
                                    );
                                },
                                on_add: move |insert_at: usize| {
                                    TagsPageOps::tag_things_thing_add(
                                        &mut input_diagram.write(),
                                        &tag_id_add,
                                    );
                                    let last = thing_count;
                                    TagsPageOps::tag_things_thing_move(
                                        &mut input_diagram.write(),
                                        &tag_id_add,
                                        last,
                                        insert_at,
                                    );
                                },
                                on_remove: move |row_index: usize| {
                                    TagsPageOps::tag_things_thing_remove(
                                        &mut input_diagram.write(),
                                        &tag_id_remove,
                                        row_index,
                                    );
                                },
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
                        TagsPageOps::tag_things_thing_add(&mut input_diagram.write(), &tag_id);
                    }
                },
                onkeydown: FieldNav::value_onkeydown(DATA_ATTR),
                "+ Add thing"
            }
        }
    }
}
