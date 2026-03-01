//! Collapsible card component for a single tag's associated things.
//!
//! Displays the tag ID, a remove button, and a list of thing IDs
//! that can be individually edited, removed, or added to.
//!
//! Supports keyboard shortcuts:
//!
//! - **ArrowRight**: expand the card (when collapsed).
//! - **ArrowLeft**: collapse the card (when expanded).
//! - **Enter**: expand + focus the first input inside the card.
//! - **Tab / Shift+Tab** (inside a field): cycle through focusable fields
//!   within the card.
//! - **Esc** (inside a field): return focus to the card wrapper.

use dioxus::{
    document,
    hooks::use_signal,
    prelude::{
        component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Key,
        ModifiersInteraction, Props,
    },
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::input_model::InputDiagram;

use crate::components::editor::{
    common::{RenameRefocus, RenameRefocusTarget, ADD_BTN, REMOVE_BTN, ROW_CLASS_SIMPLE},
    datalists::list_ids,
};

use super::{
    tag_things_card_field_keydown, tags_page_ops::TagsPageOps, COLLAPSED_HEADER_CLASS,
    FIELD_INPUT_CLASS, TAG_THINGS_CARD_CLASS,
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
    mut rename_refocus: Signal<Option<RenameRefocus>>,
) -> Element {
    let mut collapsed = use_signal(|| true);
    // Tracks which refocus target the next ID rename should use.
    // - `IdInput`: Enter or blur triggered the rename.
    // - `NextField`: forward Tab triggered the rename.
    // - `FocusParent`: Shift+Tab or Esc triggered the rename.
    let mut rename_target = use_signal(|| RenameRefocusTarget::IdInput);

    // Clone before moving into the closure so `tag_id` remains available
    // for the `rsx!` block below.
    let tag_id_for_effect = tag_id.clone();

    // After an ID rename this card is destroyed and recreated under the new
    // key. If the rename_refocus signal carries our new ID, focus the correct
    // sub-element once the DOM has settled.
    dioxus::hooks::use_effect(move || {
        let refocus = rename_refocus.read().clone();
        if let Some(RenameRefocus { new_id, target }) = refocus
            && new_id == tag_id_for_effect
        {
            rename_refocus.set(None);
            // The card was destroyed and recreated -- ensure it is
            // expanded so the user can see/interact with the fields.
            collapsed.set(false);
            let js = match target {
                RenameRefocusTarget::NextField => {
                    format!(
                        "setTimeout(() => {{\
                                let card = document.querySelector(\
                                    '[data-tag-things-card-id=\"{new_id}\"]'\
                                );\
                                if (!card) return;\
                                let items = Array.from(\
                                    card.querySelectorAll(\
                                        'input, button, [data-action=\"remove\"]'\
                                    )\
                                );\
                                if (items.length > 1) {{\
                                    items[1].focus();\
                                }} else if (items.length === 1) {{\
                                    items[0].focus();\
                                }} else {{\
                                    card.focus();\
                                }}\
                            }}, 0)"
                    )
                }
                RenameRefocusTarget::IdInput => {
                    format!(
                        "setTimeout(() => {{\
                                let card = document.querySelector(\
                                    '[data-tag-things-card-id=\"{new_id}\"]'\
                                );\
                                if (!card) return;\
                                let input = card.querySelector('input');\
                                if (input) {{\
                                    input.focus();\
                                }} else {{\
                                    card.focus();\
                                }}\
                            }}, 0)"
                    )
                }
                RenameRefocusTarget::FocusParent => {
                    format!(
                        "setTimeout(() => {{\
                                let card = document.querySelector(\
                                    '[data-tag-things-card-id=\"{new_id}\"]'\
                                );\
                                if (!card) return;\
                                card.focus();\
                            }}, 0)"
                    )
                }
            };
            document::eval(&js);
        }
    });

    let thing_count = things.len();
    let thing_suffix = if thing_count != 1 { "s" } else { "" };

    rsx! {
        div {
            class: TAG_THINGS_CARD_CLASS,
            tabindex: "0",
            "data-tag-things-card": "true",

            // === Card identity for post-rename focus === //
            "data-tag-things-card-id": "{tag_id}",

            // === Card-level keyboard shortcuts === //
            onkeydown: move |evt| {
                match evt.key() {
                    Key::ArrowRight => {
                        evt.prevent_default();
                        collapsed.set(false);
                    }
                    Key::ArrowLeft => {
                        evt.prevent_default();
                        collapsed.set(true);
                    }
                    Key::Enter => {
                        evt.prevent_default();
                        collapsed.set(false);
                        document::eval(
                            "setTimeout(() => {\
                                document.activeElement\
                                    ?.querySelector('input')\
                                    ?.focus();\
                            }, 0)"
                        );
                    }
                    _ => {}
                }
            },

            if *collapsed.read() {
                // === Collapsed summary === //
                div {
                    class: COLLAPSED_HEADER_CLASS,
                    onclick: move |_| collapsed.set(false),

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

                // Collapse toggle
                div {
                    class: "flex flex-row items-center gap-1 cursor-pointer select-none mb-1",
                    onclick: move |_| collapsed.set(true),

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
                        onkeydown: move |evt| {
                            match evt.key() {
                                Key::Tab if evt.modifiers().shift() => {
                                    rename_target.set(RenameRefocusTarget::FocusParent);
                                }
                                Key::Tab => {
                                    rename_target.set(RenameRefocusTarget::NextField);
                                }
                                Key::Escape => {
                                    rename_target.set(RenameRefocusTarget::FocusParent);
                                }
                                Key::Enter => {
                                    rename_target.set(RenameRefocusTarget::IdInput);
                                }
                                _ => {}
                            }
                            tag_things_card_field_keydown(evt);
                        },
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
                        onkeydown: move |evt| {
                            tag_things_card_field_keydown(evt);
                        },
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
                                        onkeydown: move |evt| {
                                            tag_things_card_field_keydown(evt);
                                        },
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
                                        onkeydown: move |evt| {
                                            tag_things_card_field_keydown(evt);
                                        },
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
                        onkeydown: move |evt| {
                            tag_things_card_field_keydown(evt);
                        },
                        "+ Add thing"
                    }
                }
            }
        }
    }
}
