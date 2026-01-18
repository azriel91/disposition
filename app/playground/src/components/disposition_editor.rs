use std::{
    fmt::Write,
    hash::{DefaultHasher, Hasher},
};

use dioxus::{
    hooks::{use_memo, use_signal},
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, info, rsx, Element, Props},
    signals::{Memo, ReadSignal, ReadableExt, ReadableVecExt, Signal, WritableExt},
};
use disposition::{
    input_ir_model::IrDiagramAndIssues,
    input_model::InputDiagram,
    ir_model::{
        node::{NodeId, NodeInbuilt},
        IrDiagram,
    },
    model_common::Map,
    taffy_model::{
        taffy::{self, PrintTree},
        DimensionAndLod, TaffyNodeMappings,
    },
};
use disposition_input_ir_rt::{InputToIrDiagramMapper, IrToTaffyBuilder, TaffyToSvgMapper};

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

use crate::components::{InputDiagramDiv, IrDiagramDiv, TaffyNodeMappingsDiv};

#[component]
pub fn DispositionEditor() -> Element {
    let mut status_messages: Signal<Vec<String>> = use_signal(Vec::new);
    let input_diagram_string = use_signal(|| String::from(""));
    let input_diagram: Memo<Option<InputDiagram<'static>>> = use_memo(move || {
        let mut status_messages = status_messages.write();

        // Clear this on input; there's currently no other input mechanism, so we don't
        // clear it in subsequent signals.
        status_messages.clear();

        let input_diagram_string = &*input_diagram_string.read();
        if input_diagram_string.is_empty() {
            status_messages.push(String::from("ℹ️ Enter input diagram"));

            None
        } else {
            let deserialize_start = Instant::now();
            match serde_saphyr::from_str(input_diagram_string) {
                Ok(input_diagram) => {
                    let deserialize_duration_ms =
                        Instant::now().duration_since(deserialize_start).as_millis();
                    info!("`input_diagram` deserialization took {deserialize_duration_ms} ms.");
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
    let ir_diagram: Memo<Option<IrDiagram<'static>>> = use_memo(move || {
        let mut status_messages = status_messages.write();

        let input_diagram = input_diagram.read().cloned();
        match input_diagram {
            Some(input_diagram) => {
                let input_to_ir_map_start = Instant::now();
                let input_diagram_and_issues = InputToIrDiagramMapper::map(&input_diagram);
                let input_to_ir_map_duration_ms = Instant::now()
                    .duration_since(input_to_ir_map_start)
                    .as_millis();
                info!("`InputToIrDiagramMapper::map` took {input_to_ir_map_duration_ms} ms.");
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

                Some(diagram)
            }
            None => {
                ir_diagram_string.write().clear();
                None
            }
        }
    });
    let mut taffy_node_mappings_string = use_memo(|| String::from(""));
    let taffy_node_mappings: Memo<Option<TaffyNodeMappings<'static>>> = use_memo(move || {
        let mut status_messages = status_messages.write();

        let ir_diagram = &*ir_diagram.read();
        match ir_diagram {
            Some(ir_diagram) => {
                let taffy_node_builder_start = Instant::now();
                let ir_to_taffy_builder = IrToTaffyBuilder::builder()
                    .with_ir_diagram(ir_diagram)
                    .with_dimension_and_lods(vec![DimensionAndLod::default_no_limit()])
                    .build();
                let taffy_node_builder_duration_ms = Instant::now()
                    .duration_since(taffy_node_builder_start)
                    .as_millis();
                info!("`IrToTaffyBuilder` init took {taffy_node_builder_duration_ms} ms.");

                let taffy_node_mappings_iter_result = ir_to_taffy_builder.build();
                match taffy_node_mappings_iter_result {
                    Ok(mut taffy_node_mappings_iter) => {
                        let taffy_node_mappings_start = Instant::now();
                        let taffy_node_mappings = taffy_node_mappings_iter.next()?;
                        let taffy_node_mappings_duration_ms = Instant::now()
                            .duration_since(taffy_node_mappings_start)
                            .as_millis();
                        info!("`taffy_node_mappings` generation took {taffy_node_mappings_duration_ms} ms.");

                        let mut taffy_node_mappings_string = taffy_node_mappings_string.write();
                        taffy_node_mappings_string.clear();
                        taffy_tree_fmt(&mut taffy_node_mappings_string, &taffy_node_mappings);
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
                let svg_generation_start = Instant::now();
                let svg = TaffyToSvgMapper::map(ir_diagram, taffy_node_mappings);
                let svg_generation_duration_ms = Instant::now()
                    .duration_since(svg_generation_start)
                    .as_millis();
                info!("`svg` generation took {svg_generation_duration_ms} ms.");

                svg
            })
            .unwrap_or_default()
    });

    rsx! {
        div {
            id: "disposition_editor",
            class: "
                flex
                flex-col
                lg:flex-row
                gap-2
            ",
            div {
                class: "
                    flex-1
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
            }
            object {
                class: "
                    flex-1
                ",
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
///
/// then adapted to print the disposition diagram node ID.
fn taffy_tree_fmt(buffer: &mut String, taffy_node_mappings: &TaffyNodeMappings) {
    let TaffyNodeMappings {
        taffy_tree,
        node_inbuilt_to_taffy,
        node_id_to_taffy: _,
        entity_highlighted_spans: _,
        taffy_id_to_node,
    } = taffy_node_mappings;
    let root_taffy_node_id = node_inbuilt_to_taffy
        .get(&NodeInbuilt::Root)
        .copied()
        .expect("Expected root taffy node to exist.");
    writeln!(buffer, "TREE").expect("Failed to write taffy tree to buffer");
    taffy_tree_node_fmt(
        buffer,
        taffy_tree,
        taffy_id_to_node,
        root_taffy_node_id,
        false,
        String::new(),
    );
}

/// Recursive function that prints each node in the tree
fn taffy_tree_node_fmt(
    buffer: &mut String,
    tree: &impl PrintTree,
    taffy_id_to_node: &Map<taffy::NodeId, NodeId>,
    taffy_node_id: taffy::NodeId,
    has_sibling: bool,
    lines_string: String,
) {
    let layout = &tree.get_final_layout(taffy_node_id);
    let display = taffy_id_to_node
        .get(&taffy_node_id)
        .map(|node_id| node_id.as_str())
        .unwrap_or_else(|| tree.get_debug_label(taffy_node_id));
    let num_children = tree.child_count(taffy_node_id);

    let fork_string = if has_sibling {
        "├── "
    } else {
        "└── "
    };
    writeln!(
        buffer,
        "{lines}{fork} {display} [x: {x:<4} y: {y:<4} w: {width:<4} h: {height:<4} content_w: {content_width:<4} content_h: {content_height:<4}, padding: l:{pl} r:{pr} t:{pt} b:{pb}]",
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
    )
    .expect("Failed to write taffy tree to buffer");
    let bar = if has_sibling { "│   " } else { "    " };
    let new_string = lines_string + bar;

    // Recurse into children
    tree.child_ids(taffy_node_id)
        .enumerate()
        .for_each(|(index, child)| {
            let has_sibling = index < num_children - 1;
            taffy_tree_node_fmt(
                buffer,
                tree,
                taffy_id_to_node,
                child,
                has_sibling,
                new_string.clone(),
            );
        });
}

#[component]
fn DispositionDataDivs(
    input_diagram_string: Signal<String>,
    ir_diagram_string: Memo<String>,
    ir_diagram: Memo<Option<IrDiagram<'static>>>,
    taffy_node_mappings_string: Memo<String>,
) -> Element {
    rsx! {
        div {
            id: "disposition_data_divs",
            class: "
                w-full
                flex
                flex-col
                items-center
                justify-center
                [&>*]:w-full
                lg:[&>*]:min-w-190
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
                    {
                        let mut hasher = DefaultHasher::new();
                        hasher.write(message.as_bytes());
                        let key = hasher.finish();

                        rsx! {
                            li {
                                key: "{key}",
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
    }
}
