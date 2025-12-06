//! SVG diagram generator input data model.
//!
//! The diagram input model is hand written, as an OpenAPI spec doesn't support
//! modelling certain data structures such as a Map with a particular key type.

#[macro_use]
extern crate id_newtype;

// Re-exports
// This allows consumers to not need to depend on `utoipa` manually.
#[cfg(feature = "openapi")]
pub use utoipa;

#[cfg(feature = "openapi")]
pub use crate::api_doc::ApiDoc;
pub use crate::input_diagram::InputDiagram;

pub mod common;
pub mod edge;
pub mod entity;
pub mod process;
pub mod tag;
pub mod theme;
pub mod thing;

#[cfg(feature = "openapi")]
mod api_doc;
mod input_diagram;

#[cfg(test)]
mod tests {
    const EXAMPLE_INPUT: &str = include_str!("example_input.yaml");

    use crate::InputDiagram;

    #[test]
    fn test_parse_example_input() {
        let diagram = serde_saphyr::from_str::<InputDiagram>(EXAMPLE_INPUT).unwrap();
        assert_eq!(diagram.thing_copy_text.len(), 18);
        assert_eq!(
            &[
                "proc_app_dev",
                "proc_app_release",
                "proc_i12e_region_tier_app_deploy"
            ],
            diagram
                .processes
                .iter()
                .map(|(process_id, _process_diagram)| process_id.as_str())
                .collect::<Vec<&str>>()
                .as_slice()
        );
    }
}
