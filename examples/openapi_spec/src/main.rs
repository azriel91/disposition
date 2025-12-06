use disposition::model::{utoipa::OpenApi, ApiDoc};

fn main() {
    // Unfortunately doesn't work yet.
    //
    // Fails with this error:
    //
    // ```text
    // Invalid type `RefOr::Ref` provided, cannot convert to RefOr::T<Schema>
    // ```
    //
    // <https://github.com/juhaku/utoipa/issues/663> may progress this.
    println!("{}", ApiDoc::openapi().to_pretty_json().unwrap());
}
