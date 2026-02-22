//! Rich editor pages for the disposition playground.
//!
//! Each sub-module implements one editor "page" (tab) that edits a subset of
//! the [`InputDiagram`] fields. The [`datalists`] module provides shared
//! `<datalist>` elements for autocomplete on ID fields.

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
    things_page::ThingsPage,
};

pub mod datalists;
mod processes_page;
mod tags_page;
mod text_page;
mod theme_page;
mod thing_dependencies_page;
mod things_page;
