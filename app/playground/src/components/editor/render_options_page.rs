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
    model_common::{edge::EdgeCurvature, InteractionEdgeHalo, ProcessRenderCollapse, RankDir},
};

use crate::components::editor::common::{INPUT_CLASS, LABEL_CLASS, SECTION_HEADING};

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
/// * `rank_dir`: direction that edges connect nodes.
/// * `process_render_collapse`: whether processes are rendered collapsed or
///   expanded.
/// * `dependency_edge_curvature` / `interaction_edge_curvature`: whether
///   dependency / interaction edges are drawn as smooth curves, orthogonal
///   lines, or direct (straight / curved) lines that bypass edge spacers.
/// * `interaction_edge_halo`: whether a semi-transparent halo is rendered
///   behind interaction edges.
/// * `interaction_edge_animation_millis_per_px`: how fast interaction edges
///   animate, in milliseconds of CSS animation duration per pixel travelled.
#[component]
pub fn RenderOptionsPage(input_diagram: Signal<InputDiagram<'static>>) -> Element {
    let rank_dir = input_diagram.read().render_options.rank_dir;
    let process_render_collapse = input_diagram.read().render_options.process_render_collapse;
    let dependency_edge_curvature = input_diagram
        .read()
        .render_options
        .dependency_edge_curvature;
    let interaction_edge_curvature = input_diagram
        .read()
        .render_options
        .interaction_edge_curvature;
    let interaction_edge_halo = input_diagram.read().render_options.interaction_edge_halo;
    let interaction_edge_animation_millis_per_px = input_diagram
        .read()
        .render_options
        .interaction_edge_animation_millis_per_px;

    rsx! {
        div {
            class: "flex flex-col gap-4",

            h3 { class: SECTION_HEADING, "Render Options" }
            p {
                class: LABEL_CLASS,
                "Controls how the diagram is rendered, including edge curvature \
                 and the direction edges connect nodes."
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

            // === Dependency Edge Curvature === //
            fieldset {
                class: "flex flex-col gap-1",

                legend { class: LABEL_CLASS, "Dependency Edge Curvature" }
                div {
                    class: RADIO_GROUP_CLASS,

                    div {
                        class: "flex flex-col gap-0.5",
                        label {
                            class: RADIO_LABEL_CLASS,
                            input {
                                r#type: "radio",
                                name: "dependency_edge_curvature",
                                value: "curved",
                                checked: dependency_edge_curvature == EdgeCurvature::Curved,
                                onchange: move |_| {
                                    input_diagram.write().render_options.dependency_edge_curvature =
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
                                name: "dependency_edge_curvature",
                                value: "orthogonal",
                                checked: dependency_edge_curvature == EdgeCurvature::Orthogonal,
                                onchange: move |_| {
                                    input_diagram.write().render_options.dependency_edge_curvature =
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
                                name: "dependency_edge_curvature",
                                value: "direct_straight",
                                checked: dependency_edge_curvature == EdgeCurvature::DirectStraight,
                                onchange: move |_| {
                                    input_diagram.write().render_options.dependency_edge_curvature =
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
                                name: "dependency_edge_curvature",
                                value: "direct_curved",
                                checked: dependency_edge_curvature == EdgeCurvature::DirectCurved,
                                onchange: move |_| {
                                    input_diagram.write().render_options.dependency_edge_curvature =
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

            // === Interaction Edge Curvature === //
            fieldset {
                class: "flex flex-col gap-1",

                legend { class: LABEL_CLASS, "Interaction Edge Curvature" }
                div {
                    class: RADIO_GROUP_CLASS,

                    div {
                        class: "flex flex-col gap-0.5",
                        label {
                            class: RADIO_LABEL_CLASS,
                            input {
                                r#type: "radio",
                                name: "interaction_edge_curvature",
                                value: "curved",
                                checked: interaction_edge_curvature == EdgeCurvature::Curved,
                                onchange: move |_| {
                                    input_diagram.write().render_options.interaction_edge_curvature =
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
                                name: "interaction_edge_curvature",
                                value: "orthogonal",
                                checked: interaction_edge_curvature == EdgeCurvature::Orthogonal,
                                onchange: move |_| {
                                    input_diagram.write().render_options.interaction_edge_curvature =
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
                                name: "interaction_edge_curvature",
                                value: "direct_straight",
                                checked: interaction_edge_curvature == EdgeCurvature::DirectStraight,
                                onchange: move |_| {
                                    input_diagram.write().render_options.interaction_edge_curvature =
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
                                name: "interaction_edge_curvature",
                                value: "direct_curved",
                                checked: interaction_edge_curvature == EdgeCurvature::DirectCurved,
                                onchange: move |_| {
                                    input_diagram.write().render_options.interaction_edge_curvature =
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

            // === Interaction Edge Halo === //
            fieldset {
                class: "flex flex-col gap-1",

                legend { class: LABEL_CLASS, "Interaction Edge Halo" }
                div {
                    class: RADIO_GROUP_CLASS,

                    div {
                        class: "flex flex-col gap-0.5",
                        label {
                            class: RADIO_LABEL_CLASS,
                            input {
                                r#type: "radio",
                                name: "interaction_edge_halo",
                                value: "enabled",
                                checked: interaction_edge_halo == InteractionEdgeHalo::Enabled,
                                onchange: move |_| {
                                    input_diagram.write().render_options.interaction_edge_halo =
                                        InteractionEdgeHalo::Enabled;
                                },
                            }
                            "Enabled"
                        }
                        p {
                            class: "text-xs text-gray-500 pl-6",
                            "A semi-transparent halo is rendered behind each interaction edge, \
                             sharing its path geometry, to make animated edges easier to follow. \
                             This is the default."
                        }
                    }
                    div {
                        class: "flex flex-col gap-0.5",
                        label {
                            class: RADIO_LABEL_CLASS,
                            input {
                                r#type: "radio",
                                name: "interaction_edge_halo",
                                value: "disabled",
                                checked: interaction_edge_halo == InteractionEdgeHalo::Disabled,
                                onchange: move |_| {
                                    input_diagram.write().render_options.interaction_edge_halo =
                                        InteractionEdgeHalo::Disabled;
                                },
                            }
                            "Disabled"
                        }
                        p {
                            class: "text-xs text-gray-500 pl-6",
                            "No halo is rendered behind interaction edges."
                        }
                    }
                }
            }

            // === Interaction Edge Animation Speed === //
            fieldset {
                class: "flex flex-col gap-1",

                legend { class: LABEL_CLASS, "Interaction Edge Animation Speed" }
                div {
                    class: "flex flex-col gap-0.5",
                    label {
                        class: "flex flex-row items-center gap-1.5 text-sm text-gray-200",
                        input {
                            r#type: "number",
                            class: INPUT_CLASS,
                            step: "0.1",
                            min: "0",
                            value: "{interaction_edge_animation_millis_per_px}",
                            onchange: move |evt: dioxus::events::FormEvent| {
                                if let Ok(millis_per_px) = evt.value().parse::<f64>() {
                                    input_diagram.write().render_options.interaction_edge_animation_millis_per_px =
                                        millis_per_px;
                                }
                            },
                        }
                        "ms per px"
                    }
                    p {
                        class: "text-xs text-gray-500 pl-6",
                        "Milliseconds of CSS animation duration per pixel of interaction-edge \
                         travel distance. Lower values animate faster. Default: 3.0."
                    }
                }
            }
        }
    }
}
