//! Editor page content dispatcher.
//!
//! Renders the content of the currently active editor page.

use dioxus::{
    prelude::{component, dioxus_core, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal},
};
use disposition::input_model::InputDiagram;

use crate::{
    components::editor::{
        ProcessesPage, TagsPage, TextPage, ThemeBaseStylesPage, ThemeDependenciesStylesPage,
        ThemeProcessStepStylesPage, ThemeStyleAliasesPage, ThemeTagsFocusPage,
        ThemeTypesStylesPage, ThingDependenciesPage, ThingInteractionsPage, ThingsPage,
    },
    editor_state::{EditorPage, ThingsPageUiState},
};

/// Renders the content of the currently active editor page.
#[component]
pub fn EditorPageContent(
    active_page: Signal<EditorPage>,
    input_diagram: Signal<InputDiagram<'static>>,
    things_ui_state: Signal<ThingsPageUiState>,
) -> Element {
    let page = active_page.read().clone();

    match page {
        EditorPage::Things => rsx! { ThingsPage { input_diagram, things_ui_state } },
        EditorPage::ThingDependencies => rsx! { ThingDependenciesPage { input_diagram } },
        EditorPage::ThingInteractions => rsx! { ThingInteractionsPage { input_diagram } },
        EditorPage::Processes => rsx! { ProcessesPage { input_diagram } },
        EditorPage::Tags => rsx! { TagsPage { input_diagram } },
        EditorPage::ThemeStyleAliases => rsx! { ThemeStyleAliasesPage { input_diagram } },
        EditorPage::ThemeBaseStyles => rsx! { ThemeBaseStylesPage { input_diagram } },
        EditorPage::ThemeProcessStepStyles => rsx! { ThemeProcessStepStylesPage { input_diagram } },
        EditorPage::ThemeTypesStyles => rsx! { ThemeTypesStylesPage { input_diagram } },
        EditorPage::ThemeDependenciesStyles => {
            rsx! { ThemeDependenciesStylesPage { input_diagram } }
        }
        EditorPage::ThemeTagsFocus => rsx! { ThemeTagsFocusPage { input_diagram } },
        EditorPage::Text => rsx! { TextPage { input_diagram } },
    }
}
