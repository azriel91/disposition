//! Render options editor page.
//!
//! Provides controls for editing `input_diagram.render_options`,
//! which includes edge curvature, rank direction, and process rendering
//! settings. Uses radio buttons for each option group.

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::{ReadableExt, Signal, WritableExt},
};
use disposition::{
    input_model::InputDiagram,
    model_common::{edge::EdgeCurvature, ProcessRenderCollapse, RankDir},
};

use crate::components::editor::common::{LABEL_CLASS, SECTION_HEADING};

/// CSS classes for a radio button group container.
const RADIO_GROUP_CLASS: &str = "flex flex-col gap-4";

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
/// * `dependencies_edge_curvature` / `interactions_edge_curvature`: whether
///   dependency / interaction edges are drawn as smooth curves, orthogonal
///   lines, or direct (straight / curved) lines that bypass edge spacers.
/// * `rank_dir`: direction that edges connect nodes.
/// * `process_render_collapse`: whether processes are rendered collapsed or
///   expanded.
#[component]
pub fn RenderOptionsPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    let dependencies_edge_curvature = input_diagram
        .read()
        .render_options
        .dependencies_edge_curvature;
    let interactions_edge_curvature = input_diagram
        .read()
        .render_options
        .interactions_edge_curvature;
    let rank_dir = input_diagram.read().render_options.rank_dir;
    let process_render_collapse = input_diagram.read().render_options.process_render_collapse;

    rsx! {
        div {
            class: "flex flex-col gap-4",

            h3 { class: SECTION_HEADING, "Render Options" }
            p {
                class: LABEL_CLASS,
                "Controls how the diagram is rendered, including edge curvature \
                 and the direction edges connect nodes."
            }

            // === Dependencies Edge Curvature === //
            fieldset {
                class: "flex flex-col gap-1",

                legend { class: LABEL_CLASS, "Dependencies Edge Curvature" }
                div {
                    class: RADIO_GROUP_CLASS,

                    div {
                        class: "flex flex-col gap-0.5",
                        label {
                            class: RADIO_LABEL_CLASS,
                            input {
                                r#type: "radio",
                                name: "dependencies_edge_curvature",
                                value: "curved",
                                checked: dependencies_edge_curvature == EdgeCurvature::Curved,
                                onchange: move |_| {
                                    input_diagram.write().render_options.dependencies_edge_curvature =
                                        EdgeCurvature::Curved;
                                },
                            }
                            "Curved"
                        }
                        p {
                            class: "text-xs text-gray-500 pl-6",
                            "Nodes are connected using curved lines."
                        }
                    }

                    div {
                        class: "flex flex-col gap-0.5",
                        label {
                            class: RADIO_LABEL_CLASS,
                            input {
                                r#type: "radio",
                                name: "dependencies_edge_curvature",
                                value: "orthogonal",
                                checked: dependencies_edge_curvature == EdgeCurvature::Orthogonal,
                                onchange: move |_| {
                                    input_diagram.write().render_options.dependencies_edge_curvature =
                                        EdgeCurvature::Orthogonal;
                                },
                            }
                            "Orthogonal"
                        }
                        p {
                            class: "text-xs text-gray-500 pl-6",
                            "Nodes are connected using straight lines."
                        }
                    }

                    div {
                        class: "flex flex-col gap-0.5",
                        label {
                            class: RADIO_LABEL_CLASS,
                            input {
                                r#type: "radio",
                                name: "dependencies_edge_curvature",
                                value: "direct_straight",
                                checked: dependencies_edge_curvature == EdgeCurvature::DirectStraight,
                                onchange: move |_| {
                                    input_diagram.write().render_options.dependencies_edge_curvature =
                                        EdgeCurvature::DirectStraight;
                                },
                            }
                            "Direct (Straight)"
                        }
                        p {
                            class: "text-xs text-gray-500 pl-6",
                            "Nodes are connected using straight lines drawn directly \
                             between nodes, bypassing edge spacers."
                        }
                    }

                    div {
                        class: "flex flex-col gap-0.5",
                        label {
                            class: RADIO_LABEL_CLASS,
                            input {
                                r#type: "radio",
                                name: "dependencies_edge_curvature",
                                value: "direct_curved",
                                checked: dependencies_edge_curvature == EdgeCurvature::DirectCurved,
                                onchange: move |_| {
                                    input_diagram.write().render_options.dependencies_edge_curvature =
                                        EdgeCurvature::DirectCurved;
                                },
                            }
                            "Direct (Curved)"
                        }
                        p {
                            class: "text-xs text-gray-500 pl-6",
                            "Nodes are connected using curved lines drawn directly \
                             between nodes, bypassing edge spacers."
                        }
                    }
                }
            }

            // === Interactions Edge Curvature === //
            fieldset {
                class: "flex flex-col gap-1",

                legend { class: LABEL_CLASS, "Interactions Edge Curvature" }
                div {
                    class: RADIO_GROUP_CLASS,

                    div {
                        class: "flex flex-col gap-0.5",
                        label {
                            class: RADIO_LABEL_CLASS,
                            input {
                                r#type: "radio",
                                name: "interactions_edge_curvature",
                                value: "curved",
                                checked: interactions_edge_curvature == EdgeCurvature::Curved,
                                onchange: move |_| {
                                    input_diagram.write().render_options.interactions_edge_curvature =
                                        EdgeCurvature::Curved;
                                },
                            }
                            "Curved"
                        }
                        p {
                            class: "text-xs text-gray-500 pl-6",
                            "Nodes are connected using curved lines."
                        }
                    }

                    div {
                        class: "flex flex-col gap-0.5",
                        label {
                            class: RADIO_LABEL_CLASS,
                            input {
                                r#type: "radio",
                                name: "interactions_edge_curvature",
                                value: "orthogonal",
                                checked: interactions_edge_curvature == EdgeCurvature::Orthogonal,
                                onchange: move |_| {
                                    input_diagram.write().render_options.interactions_edge_curvature =
                                        EdgeCurvature::Orthogonal;
                                },
                            }
                            "Orthogonal"
                        }
                        p {
                            class: "text-xs text-gray-500 pl-6",
                            "Nodes are connected using straight lines."
                        }
                    }

                    div {
                        class: "flex flex-col gap-0.5",
                        label {
                            class: RADIO_LABEL_CLASS,
                            input {
                                r#type: "radio",
                                name: "interactions_edge_curvature",
                                value: "direct_straight",
                                checked: interactions_edge_curvature == EdgeCurvature::DirectStraight,
                                onchange: move |_| {
                                    input_diagram.write().render_options.interactions_edge_curvature =
                                        EdgeCurvature::DirectStraight;
                                },
                            }
                            "Direct (Straight)"
                        }
                        p {
                            class: "text-xs text-gray-500 pl-6",
                            "Nodes are connected using straight lines drawn directly \
                             between nodes, bypassing edge spacers."
                        }
                    }

                    div {
                        class: "flex flex-col gap-0.5",
                        label {
                            class: RADIO_LABEL_CLASS,
                            input {
                                r#type: "radio",
                                name: "interactions_edge_curvature",
                                value: "direct_curved",
                                checked: interactions_edge_curvature == EdgeCurvature::DirectCurved,
                                onchange: move |_| {
                                    input_diagram.write().render_options.interactions_edge_curvature =
                                        EdgeCurvature::DirectCurved;
                                },
                            }
                            "Direct (Curved)"
                        }
                        p {
                            class: "text-xs text-gray-500 pl-6",
                            "Nodes are connected using curved lines drawn directly \
                             between nodes, bypassing edge spacers. This is the default."
                        }
                    }
                }
            }

            // === Rank Direction === //
            fieldset {
                class: "flex flex-col gap-1",

                legend { class: LABEL_CLASS, "Rank Direction" }
                div {
                    class: RADIO_GROUP_CLASS,

                    div {
                        class: "flex flex-col gap-0.5",
                        label {
                            class: RADIO_LABEL_CLASS,
                            input {
                                r#type: "radio",
                                name: "rank_dir",
                                value: "left_to_right",
                                checked: rank_dir == RankDir::LeftToRight,
                                onchange: move |_| {
                                    input_diagram.write().render_options.rank_dir =
                                        RankDir::LeftToRight;
                                },
                            }
                            "Left to Right"
                        }
                        p {
                            class: "text-xs text-gray-500 pl-6",
                            "e.g. Components must be built from left to right."
                        }
                    }
                    div {
                        class: "flex flex-col gap-0.5",
                        label {
                            class: RADIO_LABEL_CLASS,
                            input {
                                r#type: "radio",
                                name: "rank_dir",
                                value: "right_to_left",
                                checked: rank_dir == RankDir::RightToLeft,
                                onchange: move |_| {
                                    input_diagram.write().render_options.rank_dir =
                                        RankDir::RightToLeft;
                                },
                            }
                            "Right to Left"
                        }
                        p {
                            class: "text-xs text-gray-500 pl-6",
                            "e.g. What things can affect this thing?"
                        }
                    }
                    div {
                        class: "flex flex-col gap-0.5",
                        label {
                            class: RADIO_LABEL_CLASS,
                            input {
                                r#type: "radio",
                                name: "rank_dir",
                                value: "top_to_bottom",
                                checked: rank_dir == RankDir::TopToBottom,
                                onchange: move |_| {
                                    input_diagram.write().render_options.rank_dir =
                                        RankDir::TopToBottom;
                                },
                            }
                            "Top to Bottom"
                        }
                        p {
                            class: "text-xs text-gray-500 pl-6",
                            "e.g. Downstream effects that propagate from changes to the top."
                        }
                    }
                    div {
                        class: "flex flex-col gap-0.5",
                        label {
                            class: RADIO_LABEL_CLASS,
                            input {
                                r#type: "radio",
                                name: "rank_dir",
                                value: "bottom_to_top",
                                checked: rank_dir == RankDir::BottomToTop,
                                onchange: move |_| {
                                    input_diagram.write().render_options.rank_dir =
                                        RankDir::BottomToTop;
                                },
                            }
                            "Bottom to Top"
                        }
                        p {
                            class: "text-xs text-gray-500 pl-6",
                            "e.g. Lower layers are the foundation for higher layers."
                        }
                    }
                }
            }

            // === Process Rendering === //
            fieldset {
                class: "flex flex-col gap-1",

                legend { class: LABEL_CLASS, "Process Rendering" }
                div {
                    class: RADIO_GROUP_CLASS,

                    div {
                        class: "flex flex-col gap-0.5",
                        label {
                            class: RADIO_LABEL_CLASS,
                            input {
                                r#type: "radio",
                                name: "process_render_collapse",
                                value: "expand_when_one",
                                checked: process_render_collapse == ProcessRenderCollapse::ExpandWhenOne,
                                onchange: move |_| {
                                    input_diagram.write().render_options.process_render_collapse =
                                        ProcessRenderCollapse::ExpandWhenOne;
                                },
                            }
                            "Expand When One"
                        }
                        p {
                            class: "text-xs text-gray-500 pl-6",
                            "Expanded when there is a single process, collapsed otherwise."
                        }
                    }
                    div {
                        class: "flex flex-col gap-0.5",
                        label {
                            class: RADIO_LABEL_CLASS,
                            input {
                                r#type: "radio",
                                name: "process_render_collapse",
                                value: "expand_always",
                                checked: process_render_collapse == ProcessRenderCollapse::ExpandAlways,
                                onchange: move |_| {
                                    input_diagram.write().render_options.process_render_collapse =
                                        ProcessRenderCollapse::ExpandAlways;
                                },
                            }
                            "Expand Always"
                        }
                        p {
                            class: "text-xs text-gray-500 pl-6",
                            "Processes are always rendered fully expanded."
                        }
                    }
                    div {
                        class: "flex flex-col gap-0.5",
                        label {
                            class: RADIO_LABEL_CLASS,
                            input {
                                r#type: "radio",
                                name: "process_render_collapse",
                                value: "collapse",
                                checked: process_render_collapse == ProcessRenderCollapse::Collapse,
                                onchange: move |_| {
                                    input_diagram.write().render_options.process_render_collapse =
                                        ProcessRenderCollapse::Collapse;
                                },
                            }
                            "Collapse"
                        }
                        p {
                            class: "text-xs text-gray-500 pl-6",
                            "Processes are collapsed, expanding to reveal their steps when focused."
                        }
                    }
                }
            }
        }
    }
}
