use std::fmt::Write;

use dioxus::{
    hooks::{use_memo, use_signal},
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, info, rsx, Element, Props},
    signals::{Memo, ReadSignal, ReadableExt, ReadableVecExt, Signal, WritableExt},
};
use disposition::{
    input_ir_model::IrDiagramAndIssues,
    input_model::InputDiagram,
    ir_model::{node::NodeInbuilt, IrDiagram},
    taffy_model::{
        taffy::{self, PrintTree, TaffyTree},
        DimensionAndLod, NodeContext, TaffyNodeMappings,
    },
};
use disposition_input_ir_rt::{InputToIrDiagramMapper, IrToTaffyBuilder, TaffyToSvgMapper};

use crate::components::{InputDiagramDiv, IrDiagramDiv, TaffyNodeMappingsDiv};

#[component]
pub fn DispositionEditor() -> Element {
    let mut status_messages: Signal<Vec<String>> = use_signal(Vec::new);
    let input_diagram_string = use_signal(|| String::from(""));
    let input_diagram: Memo<Option<InputDiagram>> = use_memo(move || {
        let mut status_messages = status_messages.write();

        // Clear this on input; there's currently no other input mechanism, so we don't
        // clear it in subsequent signals.
        status_messages.clear();

        let input_diagram_string = &*input_diagram_string.read();
        info!("Running `input_diagram` signal.");
        if input_diagram_string.is_empty() {
            info!("`input_diagram_string` is empty");
            status_messages.push(String::from("ℹ️ Enter input diagram"));

            None
        } else {
            info!("Deserializing `input_diagram`");
            match serde_saphyr::from_str(input_diagram_string) {
                Ok(input_diagram) => {
                    info!("Deserialized `input_diagram`");
                    Some(input_diagram)
                }
                Err(error) => {
                    status_messages.push(String::from("⚠️ Error parsing input diagram"));
                    status_messages.push(error.to_string());
                    None
                }
            }
        }
    });
    let mut ir_diagram_string = use_memo(|| String::from(""));
    let ir_diagram: Memo<Option<IrDiagram>> = use_memo(move || {
        let mut status_messages = status_messages.write();

        let input_diagram = input_diagram.read().cloned();
        match input_diagram {
            Some(input_diagram) => {
                let input_diagram_and_issues = InputToIrDiagramMapper::map(input_diagram);
                let IrDiagramAndIssues { diagram, issues } = input_diagram_and_issues;

                if !issues.is_empty() {
                    status_messages.push(String::from(
                        "⚠️ Issues mapping input diagram to IR diagram",
                    ));
                    issues.into_iter().for_each(|issue| {
                        status_messages.push(issue.to_string());
                    });
                }

                let mut ir_diagram_string = ir_diagram_string.write();
                ir_diagram_string.clear();
                let serialization_result =
                    serde_saphyr::to_fmt_writer(&mut *ir_diagram_string, &diagram);

                let (Ok(()) | Err(())) = serialization_result.map_err(|error| {
                    status_messages.push(String::from("⚠️ Error serializing IR diagram"));
                    status_messages.push(error.to_string());
                });

                info!("Built `ir_diagram`");
                Some(diagram)
            }
            None => {
                ir_diagram_string.write().clear();
                None
            }
        }
    });
    let mut taffy_node_mappings_string = use_memo(|| String::from(""));
    let taffy_node_mappings: Memo<Option<TaffyNodeMappings>> = use_memo(move || {
        let mut status_messages = status_messages.write();

        let ir_diagram = &*ir_diagram.read();
        match ir_diagram {
            Some(ir_diagram) => {
                let ir_to_taffy_builder = IrToTaffyBuilder::builder()
                    .with_ir_diagram(ir_diagram)
                    .with_dimension_and_lods(vec![DimensionAndLod::default_lg()])
                    .build();

                let taffy_node_mappings_iter_result = ir_to_taffy_builder.build();
                match taffy_node_mappings_iter_result {
                    Ok(mut taffy_node_mappings_iter) => {
                        let taffy_node_mappings = taffy_node_mappings_iter.next()?;

                        let mut taffy_node_mappings_string = taffy_node_mappings_string.write();
                        taffy_node_mappings_string.clear();
                        let TaffyNodeMappings {
                            taffy_tree,
                            node_inbuilt_to_taffy,
                            node_id_to_taffy: _,
                            entity_highlighted_spans: _,
                        } = &taffy_node_mappings;
                        taffy_tree_fmt(
                            &mut taffy_node_mappings_string,
                            taffy_tree,
                            node_inbuilt_to_taffy
                                .get(&NodeInbuilt::Root)
                                .copied()
                                .expect("Expected root taffy node to exist."),
                        );
                        info!("Built `taffy_node_mappings`");
                        Some(taffy_node_mappings)
                    }
                    Err(error) => {
                        status_messages.push(String::from("⚠️ Error serializing IR diagram"));
                        status_messages.push(error.to_string());
                        None
                    }
                }
            }
            None => {
                taffy_node_mappings_string.write().clear();
                None
            }
        }
    });
    let svg: Memo<String> = use_memo(move || {
        let ir_diagram = ir_diagram.read();
        let taffy_node_mappings = &*taffy_node_mappings.read();
        ir_diagram
            .as_ref()
            .zip(taffy_node_mappings.clone())
            .map(|(ir_diagram, taffy_node_mappings)| {
                TaffyToSvgMapper::map(ir_diagram, taffy_node_mappings)
            })
            .unwrap_or_default()
    });

    rsx! {
        div {
            id: "disposition_editor",
            class: "
                flex
                flex-col
                gap-2
            ",
            DispositionDataDivs {
                input_diagram_string,
                ir_diagram_string,
                ir_diagram,
                taffy_node_mappings_string,
            }
            DispositionStatusMessageDiv {
                status_messages,
            }
            object {
                type: "image/svg+xml",
                data: format!("data:image/svg+xml,{}", urlencoding::encode(svg().as_str())),
            }
        }
    }
}

/// Writes a string representation of a Taffy tree to a buffer.
///
/// This is copied and modified from the `taffy::TaffyTree::print_tree` method:
///
/// <https://github.com/DioxusLabs/taffy/blob/v0.9.2/src/util/print.rs#L5>
fn taffy_tree_fmt(
    buffer: &mut String,
    taffy_tree: &TaffyTree<NodeContext>,
    root_taffy_node_id: taffy::NodeId,
) {
    writeln!(buffer, "TREE").expect("Failed to write taffy tree to buffer");
    taffy_tree_node_fmt(buffer, taffy_tree, root_taffy_node_id, false, String::new());
}

/// Recursive function that prints each node in the tree
fn taffy_tree_node_fmt(
    buffer: &mut String,
    tree: &impl PrintTree,
    node_id: taffy::NodeId,
    has_sibling: bool,
    lines_string: String,
) {
    let layout = &tree.get_final_layout(node_id);
    let display = tree.get_debug_label(node_id);
    let num_children = tree.child_count(node_id);

    let fork_string = if has_sibling {
        "├── "
    } else {
        "└── "
    };
    writeln!(
        buffer,
        "{lines}{fork} {display} [x: {x:<4} y: {y:<4} w: {width:<4} h: {height:<4} content_w: {content_width:<4} content_h: {content_height:<4}, padding: l:{pl} r:{pr} t:{pt} b:{pb}] ({key:?})",
        lines = lines_string,
        fork = fork_string,
        display = display,
        x = layout.location.x,
        y = layout.location.y,
        width = layout.size.width,
        height = layout.size.height,
        content_width = layout.content_size.width,
        content_height = layout.content_size.height,

        // `border: l:{bl} r:{br} t:{bt} b:{bb}`
        //
        // bl = layout.border.left,
        // br = layout.border.right,
        // bt = layout.border.top,
        // bb = layout.border.bottom,

        pl = layout.padding.left,
        pr = layout.padding.right,
        pt = layout.padding.top,
        pb = layout.padding.bottom,
        key = node_id,
    )
    .expect("Failed to write taffy tree to buffer");
    let bar = if has_sibling { "│   " } else { "    " };
    let new_string = lines_string + bar;

    // Recurse into children
    tree.child_ids(node_id)
        .enumerate()
        .for_each(|(index, child)| {
            let has_sibling = index < num_children - 1;
            taffy_tree_node_fmt(buffer, tree, child, has_sibling, new_string.clone());
        });
}

#[component]
fn DispositionDataDivs(
    input_diagram_string: Signal<String>,
    ir_diagram_string: Memo<String>,
    ir_diagram: Memo<Option<IrDiagram>>,
    taffy_node_mappings_string: Memo<String>,
) -> Element {
    rsx! {
        div {
            id: "disposition_data_divs",
            class: "
                w-full
                flex
                flex-row
                flex-wrap
                items-center
                justify-center
            ",
            InputDiagramDiv { input_diagram_string }
            IrDiagramDiv { ir_diagram_string }
            TaffyNodeMappingsDiv { taffy_node_mappings_string }
        }
    }
}

#[component]
fn DispositionStatusMessageDiv(status_messages: ReadSignal<Vec<String>>) -> Element {
    rsx! {
        div {
            id: "disposition_status_message_div",
            class: "
                w-full
                flex
                flex-col
                gap-1
            ",
            h3 {
                class: "
                    text-sm
                    font-bold
                    text-gray-300
                ",
                "Status Message"
            }
            DispositionStatusMessage {
                status_messages,
            }
        }
    }
}

#[component]
fn DispositionStatusMessage(status_messages: ReadSignal<Vec<String>>) -> Element {
    rsx! {
        div {
            id: "disposition_status_message",
            class: "
                rounded-lg
                border
                border-gray-300
                bg-gray-800
                font-mono
                p-2
                select-text
            ",
            ul {
                class: "
                    list-disc
                    list-inside
                ",
                for message in status_messages.iter() {
                    li {
                        class: "
                            text-sm
                            text-gray-300
                        ",
                        "{message}"
                    }
                }
            }
        }
    }
}
