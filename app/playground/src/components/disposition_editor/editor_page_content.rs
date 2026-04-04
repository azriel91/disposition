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
        EntityTypesPage, ProcessesPage, RenderOptionsPage, TagsPage, TextPage, ThemeBaseStylesPage,
        ThemeDependenciesStylesPage, ThemeProcessStepStylesPage, ThemeStyleAliasesPage,
        ThemeTagsFocusPage, ThemeTypesStylesPage, ThingCopyTextPage, ThingDependenciesPage,
        ThingEntityDescsPage, ThingEntityTooltipsPage, ThingInteractionsPage, ThingLayoutPage,
        ThingNamesPage,
    },
    editor_state::{EditorPage, EditorPageTheme, EditorPageThing},
};

/// Renders the content of the currently active editor page.
#[component]
pub fn EditorPageContent(
    active_page: Signal<EditorPage>,
    input_diagram: Signal<InputDiagram<'static>>,
) -> Element {
    let page = active_page.read().clone();

    match page {
        EditorPage::Thing(sub) => match sub {
            EditorPageThing::Names => rsx! { ThingNamesPage { input_diagram } },
            EditorPageThing::CopyText => rsx! { ThingCopyTextPage { input_diagram } },
            EditorPageThing::EntityDescs => rsx! { ThingEntityDescsPage { input_diagram } },
            EditorPageThing::EntityTooltips => rsx! { ThingEntityTooltipsPage { input_diagram } },
        },
        EditorPage::ThingLayout => rsx! { ThingLayoutPage { input_diagram } },
        EditorPage::ThingDependencies => rsx! { ThingDependenciesPage { input_diagram } },
        EditorPage::ThingInteractions => rsx! { ThingInteractionsPage { input_diagram } },
        EditorPage::Processes => rsx! { ProcessesPage { input_diagram } },
        EditorPage::Tags => rsx! { TagsPage { input_diagram } },
        EditorPage::EntityTypes => rsx! { EntityTypesPage { input_diagram } },
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
