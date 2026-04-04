//! Rich editor pages for the disposition playground.
//!
//! Each sub-module implements one editor "page" (tab) that edits a subset of
//! the [`InputDiagram`](disposition::input_model::InputDiagram) fields. The
//! [`datalists`] module provides shared `<datalist>` elements for autocomplete
//! on ID fields.

pub use self::{
    datalists::EditorDataLists,
    entity_types_page::EntityTypesPage,
    processes_page::ProcessesPage,
    render_options_page::RenderOptionsPage,
    tags_page::TagsPage,
    text_page::TextPage,
    theme_page::{
        ThemeBaseStylesPage, ThemeDependenciesStylesPage, ThemeProcessStepStylesPage,
        ThemeStyleAliasesPage, ThemeTagsFocusPage, ThemeTypesStylesPage,
    },
    thing_dependencies_page::{ThingDependenciesPage, ThingInteractionsPage},
    thing_layout_page::ThingLayoutPage,
    things_page::{
        ThingCopyTextPage, ThingEntityDescsPage, ThingEntityTooltipsPage, ThingNamesPage,
    },
};

pub mod datalists;

pub(crate) mod common;
pub(crate) mod id_value_row;
pub(crate) mod keyboard_nav;
pub(crate) mod reorderable;
pub(crate) mod theme_styles_editor;

pub(crate) mod entity_types_page;
pub(crate) mod processes_page;
pub(crate) mod render_options_page;
pub(crate) mod tags_page;
pub(crate) mod text_page;
pub(crate) mod theme_page;
pub(crate) mod thing_dependencies_page;
pub(crate) mod thing_layout_page;
pub(crate) mod things_page;
