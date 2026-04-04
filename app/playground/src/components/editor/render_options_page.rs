//! Render options editor page.
//!
//! Provides controls for editing `input_diagram.render_options`,
//! which includes edge curvature and rank direction settings.
//! Uses radio buttons for each option group.

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::{
    input_model::InputDiagram,
    model_common::{edge::EdgeCurvature, RankDir},
};

use crate::components::editor::common::{LABEL_CLASS, SECTION_HEADING};

/// CSS classes for a radio button group container.
const RADIO_GROUP_CLASS: &str = "flex flex-row gap-4";

/// CSS classes for an individual radio label (clickable wrapper).
const RADIO_LABEL_CLASS: &str = "\
    flex flex-row items-center gap-1.5 \
    text-sm text-gray-200 \
    cursor-pointer\
";

/// The **Render Options** editor page.
///
/// Allows the user to configure:
///
/// * `edge_curvature` -- whether edges are drawn as smooth curves or orthogonal
///   lines.
/// * `rank_dir` -- whether edges connect nodes vertically or horizontally.
#[component]
pub fn RenderOptionsPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    let edge_curvature = input_diagram.read().render_options.edge_curvature;
    let rank_dir = input_diagram.read().render_options.rank_dir;

    rsx! {
        div {
            class: "flex flex-col gap-4",

            h3 { class: SECTION_HEADING, "Render Options" }
            p {
                class: LABEL_CLASS,
                "Controls how the diagram is rendered, including edge curvature \
                 and the direction edges connect nodes."
            }

            // === Edge Curvature === //
            fieldset {
                class: "flex flex-col gap-1",

                legend { class: LABEL_CLASS, "Edge Curvature" }
                div {
                    class: RADIO_GROUP_CLASS,

                    label {
                        class: RADIO_LABEL_CLASS,
                        input {
                            r#type: "radio",
                            name: "edge_curvature",
                            value: "curved",
                            checked: edge_curvature == EdgeCurvature::Curved,
                            onchange: move |_| {
                                input_diagram.write().render_options.edge_curvature =
                                    EdgeCurvature::Curved;
                            },
                        }
                        "Curved"
                    }
                    label {
                        class: RADIO_LABEL_CLASS,
                        input {
                            r#type: "radio",
                            name: "edge_curvature",
                            value: "orthogonal",
                            checked: edge_curvature == EdgeCurvature::Orthogonal,
                            onchange: move |_| {
                                input_diagram.write().render_options.edge_curvature =
                                    EdgeCurvature::Orthogonal;
                            },
                        }
                        "Orthogonal"
                    }
                }
            }

            // === Rank Direction === //
            fieldset {
                class: "flex flex-col gap-1",

                legend { class: LABEL_CLASS, "Rank Direction" }
                div {
                    class: RADIO_GROUP_CLASS,

                    label {
                        class: RADIO_LABEL_CLASS,
                        input {
                            r#type: "radio",
                            name: "rank_dir",
                            value: "vertical",
                            checked: rank_dir == RankDir::Vertical,
                            onchange: move |_| {
                                input_diagram.write().render_options.rank_dir = RankDir::Vertical;
                            },
                        }
                        "Vertical"
                    }
                    label {
                        class: RADIO_LABEL_CLASS,
                        input {
                            r#type: "radio",
                            name: "rank_dir",
                            value: "horizontal",
                            checked: rank_dir == RankDir::Horizontal,
                            onchange: move |_| {
                                input_diagram.write().render_options.rank_dir = RankDir::Horizontal;
                            },
                        }
                        "Horizontal"
                    }
                }
            }
        }
    }
}
