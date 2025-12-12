//! OpenAPI documentation for the disposition_ir_model crate.

use utoipa::OpenApi;

/// OpenAPI documentation for the intermediate representation types.
#[derive(OpenApi)]
#[openapi(
    components(schemas(
        // Common types
        disposition_model_common::Id,
        disposition_model_common::edge::EdgeGroupId,
        disposition_model_common::edge::EdgeId,
        disposition_model_common::theme::Css,
        // Entity types
        crate::entity::EntityDescs,
        crate::entity::EntityTailwindClasses,
        crate::entity::EntityTypeId,
        crate::entity::EntityTypes,
        // Node types
        crate::node::NodeId,
        crate::node::NodeNames,
        crate::node::NodeCopyText,
        crate::node::NodeHierarchy,
        // Edge types
        crate::edge::Edge,
        crate::edge::EdgeGroup,
        crate::edge::EdgeGroups,
        // Layout types
        crate::layout::FlexDirection,
        crate::layout::FlexLayout,
        crate::layout::NodeLayout,
        crate::layout::NodeLayouts,
        // Root type
        crate::IrDiagram,
    ))
)]
pub struct ApiDoc;
