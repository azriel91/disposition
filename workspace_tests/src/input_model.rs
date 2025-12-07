const EXAMPLE_INPUT: &str = include_str!("example_input.yaml");

use disposition::model::InputDiagram;

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
