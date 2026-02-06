//! Contains all shared components for our app.
//!
//! The components module contains all shared components for our app. Components
//! are the building blocks of dioxus apps. They can be used to defined common
//! UI elements like buttons, forms, and modals.

pub use self::{
    disposition_editor::DispositionEditor, input_diagram_div::InputDiagramDiv,
    ir_diagram_div::IrDiagramDiv, svg_elements_div::SvgElementsDiv,
    taffy_node_mappings_div::TaffyNodeMappingsDiv,
};

mod disposition_editor;
mod input_diagram_div;
mod ir_diagram_div;
mod svg_elements_div;
mod taffy_node_mappings_div;
