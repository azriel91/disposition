//! Entity editor page.
//!
//! Provides sub-pages for:
//! - Entity Types (`entity_types`: `Id` -> `Set<EntityType>`)
//! - Entity Tooltips (`entity_tooltips`: `Id` -> tooltip)

use dioxus::{
    prelude::{component, dioxus_core, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal},
};
use disposition::input_model::InputDiagram;

use crate::{
    components::editor::{EntityTooltipsPage, EntityTypesPage},
    editor_state::{EditorPage, EditorPageEntity},
};

/// The **Entity** editor page.
///
/// Dispatches to the active entity sub-page:
/// - [`EntityTypesPage`]: entity type assignments for common styling.
/// - [`EntityTooltipsPage`]: tooltip text shown on hover for nodes and edges.
#[component]
pub fn EntityPage(
    active_page: Signal<EditorPage>,
    input_diagram: Signal<InputDiagram<'static>>,
) -> Element {
    let page = active_page.read().clone();
    match page {
        EditorPage::Entity(EditorPageEntity::EntityTypes) => {
            rsx! { EntityTypesPage { input_diagram } }
        }
        EditorPage::Entity(EditorPageEntity::EntityTooltips) => {
            rsx! { EntityTooltipsPage { input_diagram } }
        }
        _ => rsx! {},
    }
}
