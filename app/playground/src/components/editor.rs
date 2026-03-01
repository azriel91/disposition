//! Rich editor pages for the disposition playground.
//!
//! Each sub-module implements one editor "page" (tab) that edits a subset of
//! the [`InputDiagram`](disposition::input_model::InputDiagram) fields. The
//! [`datalists`] module provides shared `<datalist>` elements for autocomplete
//! on ID fields.

pub use self::{
    datalists::EditorDataLists,
    processes_page::ProcessesPage,
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
pub(crate) mod key_value_row_container;
pub(crate) mod theme_styles_editor;

mod processes_page;
mod tags_page;
mod text_page;
mod theme_page;
mod thing_dependencies_page;
mod thing_layout_page;
mod things_page;
