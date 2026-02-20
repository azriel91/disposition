//! OpenAPI documentation for the disposition_ir_model crate.

use utoipa::OpenApi;

/// OpenAPI documentation for the intermediate representation types.
#[derive(OpenApi)]
#[openapi(components(schemas(
    crate::SvgElements,
    crate::SvgEdgeInfo,
    crate::SvgNodeInfo,
    crate::SvgNodeInfoCircle,
    crate::SvgProcessInfo,
    crate::SvgTextSpan
)))]
pub struct ApiDoc;
