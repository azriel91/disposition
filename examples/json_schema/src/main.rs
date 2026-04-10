#[cfg(all(feature = "schemars", not(feature = "test")))]
use disposition::{input_model::InputDiagram, schemars};

fn main() {
    #[cfg(all(feature = "schemars", not(feature = "test")))]
    {
        let schema = schemars::schema_for!(InputDiagram);
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
    }

    #[cfg(any(not(feature = "schemars"), feature = "test"))]
    eprintln!("Please enable the `\"schemars\"` feature to run this example.")
}
