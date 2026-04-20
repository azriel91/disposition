use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Key, Props},
    signals::{ReadableExt, Signal, WritableExt},
};

use crate::{
    components::disposition_editor::editor_tab_bar::{TAB_ACTIVE, TAB_CLASS, TAB_INACTIVE},
    editor_state::{EditorPage, EditorPageEntity, EditorPageTheme, EditorPageThing},
};

#[component]
pub(crate) fn EditorTabBarTabs(active_page: Signal<EditorPage>) -> Element {
    let current_page = active_page.read().clone();
    let top_level = EditorPage::top_level_pages();

    rsx! {
        for (tab_idx, entry) in top_level.iter().enumerate() {
            {
                let is_active = entry.same_top_level(&current_page);
                let css = format!(
                    "{TAB_CLASS} {}",
                    if is_active { TAB_ACTIVE } else { TAB_INACTIVE }
                );
                // Only the active tab is in the tab order; others use
                // tabindex="-1" and are reachable via arrow keys.
                let tab_index = if is_active { "0" } else { "-1" };
                let entry_clone = entry.clone();
                let tab_idx_str = tab_idx.to_string();

                rsx! {
                    span {
                        key: "{entry.top_level_label()}",
                        role: "tab",
                        tabindex: "{tab_index}",
                        "aria-selected": if is_active { "true" } else { "false" },
                        "data-top-level-index": "{tab_idx_str}",
                        class: "{css}",
                        onclick: {
                            let entry = entry_clone.clone();
                            move |_| {
                                editor_tab_bar_top_level_activate(active_page, &entry);
                            }
                        },
                        onkeydown: {
                            let entry = entry_clone.clone();
                            move |evt| {
                                let activate = match evt.key() {
                                    Key::Enter => true,
                                    Key::Character(ref c) if c == " " => true,
                                    _ => false,
                                };
                                if activate {
                                    evt.prevent_default();
                                    editor_tab_bar_top_level_activate(active_page, &entry);
                                }
                            }
                        },
                        "{entry.top_level_label()}"
                    }
                }
            }
        }
    }
}

/// Activates a top-level tab entry.
///
/// For grouped tabs (`Thing(_)` or `Theme(_)`), only switches to the
/// group's default sub-page if the user is not already on a page within
/// that group. This preserves the user's sub-tab selection when
/// re-clicking the same group tab.
fn editor_tab_bar_top_level_activate(mut active_page: Signal<EditorPage>, entry: &EditorPage) {
    match entry {
        EditorPage::Thing(_) => {
            // If already on a thing page, stay there;
            // otherwise default to Thing::Names.
            if !active_page.peek().is_thing() {
                active_page.set(EditorPage::Thing(EditorPageThing::default()));
            }
        }
        EditorPage::Entity(_) => {
            // If already on an entity page, stay there;
            // otherwise default to Entity::EntityTypes.
            if !active_page.peek().is_entity() {
                active_page.set(EditorPage::Entity(EditorPageEntity::default()));
            }
        }
        EditorPage::Theme(_) => {
            // If already on a theme page, stay there;
            // otherwise default to Theme::StyleAliases.
            if !active_page.peek().is_theme() {
                active_page.set(EditorPage::Theme(EditorPageTheme::default()));
            }
        }
        other => {
            active_page.set(other.clone());
        }
    }
}
