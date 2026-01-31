//! OpenAPI documentation for the disposition_ir_model crate.

use utoipa::OpenApi;

/// OpenAPI documentation for the intermediate representation types.
#[derive(OpenApi)]
#[openapi(components(schemas(crate::SvgElements, crate::SvgEdgeInfo, crate::SvgNodeInfo)))]
pub struct ApiDoc;
