//! Dropdown selector for loading example diagrams into the editor.
//!
//! Renders a `<select>` element listing all [`ExampleDiagram`] variants. When
//! the user picks an example, the YAML is deserialized and written into the
//! [`InputDiagram`] signal.

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{Signal, WritableExt},
};
use disposition::input_model::InputDiagram;

use crate::example_diagrams::ExampleDiagram;

/// CSS classes for the select element.
const SELECT_CLASS: &str = "\
    rounded \
    px-2 py-1 \
    text-sm \
    font-semibold \
    cursor-pointer \
    select-none \
    bg-gray-700 \
    hover:bg-gray-600 \
    text-gray-200 \
    border \
    border-gray-600 \
    focus:outline-none \
    focus:border-blue-400\
";

/// A `<select>` dropdown that lets the user pick an example diagram.
///
/// When an example is selected, its YAML content is deserialized and the
/// resulting [`InputDiagram`] is written into the provided signal.
#[component]
pub fn ExampleDiagramSelect(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    rsx! {
        select {
            class: SELECT_CLASS,
            title: "Load an example diagram",
            "aria-label": "Load example diagram",
            onchange: move |evt: dioxus::events::FormEvent| {
                let value = evt.value();
                if let Ok(index) = value.parse::<usize>() {
                    if let Some(example) = ExampleDiagram::from_index(index) {
                        let yaml = example.yaml();
                        if let Ok(diagram) = serde_saphyr::from_str::<InputDiagram<'static>>(yaml) {
                            input_diagram.set(diagram);
                        }
                    }
                }
            },

            // Placeholder option that is always the visible default.
            option {
                value: "",
                disabled: true,
                selected: true,
                "Examples..."
            }

            for (index, example) in ExampleDiagram::ALL.iter().enumerate() {
                option {
                    value: "{index}",
                    "{example.label()}"
                }
            }
        }
    }
}
