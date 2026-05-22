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
        EntityDescsPage, EntityTooltipsPage, EntityTypesPage, ProcessesPage, RenderOptionsPage,
        TagsPage, TextPage, ThemeBaseStylesPage, ThemeDependenciesStylesPage,
        ThemeProcessStepStylesPage, ThemeStyleAliasesPage, ThemeTagsFocusPage,
        ThemeTypesStylesPage, ThingCopyTextPage, ThingDependenciesPage, ThingInteractionsPage,
        ThingLayoutPage, ThingNamesPage,
    },
    editor_state::{EditorPage, EditorPageEntity, EditorPageTheme, EditorPageThing},
};

/// Renders the content of the currently active editor page.
#[component]
pub fn EditorPageContent(
    active_page: Signal<EditorPage>,
    input_diagram: Signal<InputDiagram<'static>>,
) -> Element {
    let page = active_page.read().clone();

    match page {
        EditorPage::Thing(editor_page_thing) => match editor_page_thing {
            EditorPageThing::Names => rsx! { ThingNamesPage { input_diagram } },
            EditorPageThing::CopyText => rsx! { ThingCopyTextPage { input_diagram } },
        },
        EditorPage::ThingLayout => rsx! { ThingLayoutPage { input_diagram } },
        EditorPage::ThingDependencies => rsx! { ThingDependenciesPage { input_diagram } },
        EditorPage::ThingInteractions => rsx! { ThingInteractionsPage { input_diagram } },
        EditorPage::Processes => rsx! { ProcessesPage { input_diagram } },
        EditorPage::Tags => rsx! { TagsPage { input_diagram } },
        EditorPage::Entity(editor_page_entity) => match editor_page_entity {
            EditorPageEntity::Types => rsx! { EntityTypesPage { input_diagram } },
            EditorPageEntity::Tooltips => rsx! { EntityTooltipsPage { input_diagram } },
            EditorPageEntity::Descs => rsx! { EntityDescsPage { input_diagram } },
        },
        EditorPage::RenderOptions => rsx! { RenderOptionsPage { input_diagram } },
        EditorPage::Theme(sub) => match sub {
            EditorPageTheme::BaseStyles => rsx! { ThemeBaseStylesPage { input_diagram } },
            EditorPageTheme::TypesStyles => rsx! { ThemeTypesStylesPage { input_diagram } },
            EditorPageTheme::ProcessStepStyles => {
                rsx! { ThemeProcessStepStylesPage { input_diagram } }
            }
            EditorPageTheme::DependenciesStyles => {
                rsx! { ThemeDependenciesStylesPage { input_diagram } }
            }
            EditorPageTheme::TagsFocus => rsx! { ThemeTagsFocusPage { input_diagram } },
            EditorPageTheme::StyleAliases => rsx! { ThemeStyleAliasesPage { input_diagram } },
        },
        EditorPage::Text => rsx! { TextPage { input_diagram } },
    }
}
