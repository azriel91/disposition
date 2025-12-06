//! OpenAPI documentation for the disposition_ir crate.

use utoipa::OpenApi;

/// OpenAPI documentation for the intermediate representation types.
#[derive(OpenApi)]
#[openapi(
    components(schemas(
        // Common types
        crate::common::Id,
        // Entity types
        crate::entity::EntityTypeId,
        crate::entity::EntityTypes,
        crate::entity::EntityDescs,
        // Node types
        crate::node::NodeId,
        crate::node::NodeNames,
        crate::node::NodeCopyText,
        crate::node::NodeHierarchy,
        crate::node::TailwindClasses,
        // Edge types
        crate::edge::EdgeId,
        crate::edge::EdgeGroupId,
        crate::edge::Edge,
        crate::edge::EdgeGroup,
        crate::edge::EdgeGroups,
        // Layout types
        crate::layout::Css,
        crate::layout::FlexDirection,
        crate::layout::FlexLayout,
        crate::layout::NodeLayout,
        crate::layout::NodeLayouts,
        // Root type
        crate::IrDiagram,
    ))
)]
pub struct ApiDoc;
