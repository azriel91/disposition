use crate::IrDiagramAndIssues;

/// Structure that represents the OpenAPI documentation for the Disposition API.
#[derive(utoipa::OpenApi)]
#[openapi(components(schemas(IrDiagramAndIssues,)))]
pub struct ApiDoc;
