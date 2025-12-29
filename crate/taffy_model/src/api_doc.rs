//! OpenAPI documentation for the disposition_ir_model crate.

use utoipa::OpenApi;

/// OpenAPI documentation for the intermediate representation types.
#[derive(OpenApi)]
#[openapi(components(schemas(
    crate::DiagramLod,
    crate::DimensionAndLod,
    crate::Dimension,
    crate::NodeContext
)))]
pub struct ApiDoc;
