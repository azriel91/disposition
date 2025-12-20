#[cfg(all(feature = "openapi", not(feature = "test")))]
use disposition::input_model::{utoipa::OpenApi, ApiDoc};

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
    #[cfg(all(feature = "openapi", not(feature = "test")))]
    println!("{}", ApiDoc::openapi().to_pretty_json().unwrap());

    #[cfg(any(not(feature = "openapi"), feature = "test"))]
    eprintln!("Please enable the `\"openapi\"` feature to run this example.")
}
