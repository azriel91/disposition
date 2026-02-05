use std::{
    fmt::Write,
    hash::{DefaultHasher, Hasher},
};

use dioxus::{
    hooks::{use_memo, use_signal},
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, info, rsx, Element, Props},
    signals::{Memo, ReadableExt, ReadableVecExt, Signal},
};
use disposition::{
    input_ir_model::IrDiagramAndIssues,
    input_model::InputDiagram,
    ir_model::{
        node::{NodeId, NodeInbuilt},
        IrDiagram,
    },
    model_common::Map,
    svg_model::SvgElements,
    taffy_model::{
        taffy::{self, PrintTree},
        DimensionAndLod, TaffyNodeMappings,
    },
};
use disposition_input_ir_rt::{
    InputDiagramMerger, InputToIrDiagramMapper, IrToTaffyBuilder, SvgElementsToSvgMapper,
    TaffyToSvgElementsMapper,
};

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

use crate::components::{InputDiagramDiv, IrDiagramDiv, SvgElementsDiv, TaffyNodeMappingsDiv};

#[component]
#[allow(clippy::type_complexity)] // Maybe reduce complexity for `Memo<_>` types when we refactor this.
pub fn DispositionEditor() -> Element {
    let input_diagram_string = use_signal(|| String::from(""));

    // Parse input diagram string into InputDiagram
    let input_diagram: Memo<Result<InputDiagram<'static>, Vec<String>>> = use_memo(move || {
        let input_diagram_string = &*input_diagram_string.read();
        if input_diagram_string.is_empty() {
            Err(vec![String::from("ℹ️ Enter input diagram")])
        } else {
            let deserialize_start = Instant::now();
            let input_diagram = serde_saphyr::from_str(input_diagram_string).map_err(|error| {
                vec![
                    String::from("⚠️ Error parsing input diagram"),
                    error.to_string(),
                ]
            });

            let deserialize_duration_ms =
                Instant::now().duration_since(deserialize_start).as_millis();
            info!("`input_diagram` deserialization took {deserialize_duration_ms} ms.");

            input_diagram
        }
    });

    // Map InputDiagram to IrDiagram
    // Ok variant contains (diagram, warnings), Err variant contains errors
    let ir_diagram: Memo<Result<(IrDiagram<'static>, Vec<String>), Vec<String>>> =
        use_memo(move || {
            let input_diagram = &*input_diagram.read();
            match input_diagram {
                Ok(input_diagram) => {
                    let input_diagram_merge_start = Instant::now();
                    let input_diagram_merged =
                        InputDiagramMerger::merge(InputDiagram::base(), input_diagram);
                    let input_diagram_merge_duration_ms = Instant::now()
                        .duration_since(input_diagram_merge_start)
                        .as_millis();
                    info!("`InputDiagramMerger::merge` took {input_diagram_merge_duration_ms} ms.");

                    let input_to_ir_map_start = Instant::now();
                    let input_diagram_and_issues =
                        InputToIrDiagramMapper::map(&input_diagram_merged);
                    let input_to_ir_map_duration_ms = Instant::now()
                        .duration_since(input_to_ir_map_start)
                        .as_millis();
                    info!("`InputToIrDiagramMapper::map` took {input_to_ir_map_duration_ms} ms.");
                    let IrDiagramAndIssues { diagram, issues } = input_diagram_and_issues;

                    let warnings = if !issues.is_empty() {
                        let mut msgs = vec![String::from(
                            "⚠️ Issues mapping input diagram to IR diagram",
                        )];
                        msgs.extend(issues.into_iter().map(|issue| issue.to_string()));
                        msgs
                    } else {
                        vec![]
                    };

                    Ok((diagram, warnings))
                }
                Err(_) => Err(vec![]), // Previous step failed, nothing to do
            }
        });

    // Serialize IrDiagram to string
    let ir_diagram_string: Memo<String> = use_memo(move || {
        let ir_diagram = &*ir_diagram.read();
        match ir_diagram {
            Ok((diagram, _)) => {
                let mut buffer = String::new();
                match serde_saphyr::to_fmt_writer(&mut buffer, diagram) {
                    Ok(()) => buffer,
                    Err(error) => format!("⚠️ Error serializing IR diagram: {}", error),
                }
            }
            Err(_) => String::new(),
        }
    });

    // Build TaffyNodeMappings from IrDiagram
    let taffy_node_mappings: Memo<Result<TaffyNodeMappings<'static>, Vec<String>>> = use_memo(
        move || {
            let ir_diagram = &*ir_diagram.read();
            match ir_diagram {
                Ok((ir_diagram, _)) => {
                    let ir_to_taffy_builder = IrToTaffyBuilder::builder()
                        .with_ir_diagram(ir_diagram)
                        .with_dimension_and_lods(vec![DimensionAndLod::default_no_limit()])
                        .build();

                    let taffy_node_mappings_iter_result = ir_to_taffy_builder.build();
                    match taffy_node_mappings_iter_result {
                        Ok(mut taffy_node_mappings_iter) => {
                            let taffy_node_mappings_start = Instant::now();
                            match taffy_node_mappings_iter.next() {
                                Some(taffy_node_mappings) => {
                                    let taffy_node_mappings_duration_ms = Instant::now()
                                        .duration_since(taffy_node_mappings_start)
                                        .as_millis();
                                    info!("`taffy_node_mappings` generation took {taffy_node_mappings_duration_ms} ms.");
                                    Ok(taffy_node_mappings)
                                }
                                None => {
                                    Err(vec![String::from("⚠️ No taffy node mappings generated")])
                                }
                            }
                        }
                        Err(error) => Err(vec![
                            String::from("⚠️ Error building taffy tree"),
                            error.to_string(),
                        ]),
                    }
                }
                Err(_) => Err(vec![]), // Previous step failed
            }
        },
    );

    // Format TaffyNodeMappings to string
    let taffy_node_mappings_string: Memo<String> = use_memo(move || {
        let taffy_node_mappings = &*taffy_node_mappings.read();
        match taffy_node_mappings {
            Ok(taffy_node_mappings) => {
                let mut buffer = String::new();
                taffy_tree_fmt(&mut buffer, taffy_node_mappings);
                buffer
            }
            Err(_) => String::new(),
        }
    });

    // Map to SvgElements
    let svg_elements: Memo<Result<SvgElements, Vec<String>>> = use_memo(move || {
        let ir_diagram = &*ir_diagram.read();
        let taffy_node_mappings = &*taffy_node_mappings.read();

        match (ir_diagram, taffy_node_mappings) {
            (Ok((ir_diagram, _)), Ok(taffy_node_mappings)) => {
                let svg_elements_map_start = Instant::now();
                let svg_elements = TaffyToSvgElementsMapper::map(ir_diagram, taffy_node_mappings);
                let svg_elements_map_duration_ms = Instant::now()
                    .duration_since(svg_elements_map_start)
                    .as_millis();
                info!("`TaffyToSvgElementsMapper::map` took {svg_elements_map_duration_ms} ms.");
                Ok(svg_elements)
            }
            _ => Err(vec![]), // Previous steps failed
        }
    });

    // Serialize SvgElements to string
    let svg_elements_string: Memo<String> = use_memo(move || {
        let svg_elements = &*svg_elements.read();
        match svg_elements {
            Ok(svg_elements) => {
                let mut buffer = String::new();
                match serde_saphyr::to_fmt_writer(&mut buffer, svg_elements) {
                    Ok(()) => buffer,
                    Err(error) => format!("⚠️ Error serializing SVG elements: {}", error),
                }
            }
            Err(_) => String::new(),
        }
    });

    // Generate final SVG string
    let svg: Memo<String> = use_memo(move || {
        let svg_elements = &*svg_elements.read();
        let svg_generation_start = Instant::now();
        let svg = match svg_elements {
            Ok(svg_elements) => SvgElementsToSvgMapper::map(svg_elements),
            Err(_) => String::new(),
        };
        let svg_generation_duration_ms = Instant::now()
            .duration_since(svg_generation_start)
            .as_millis();
        info!("`svg` generation took {svg_generation_duration_ms} ms.");
        svg
    });

    // Collect all status messages from the processing pipeline
    let status_messages: Memo<Vec<String>> = use_memo(move || {
        let mut messages = Vec::new();

        // Collect from input_diagram
        if let Err(errors) = &*input_diagram.read() {
            messages.extend(errors.iter().cloned());
        }

        // Collect from ir_diagram (both warnings and errors)
        match &*ir_diagram.read() {
            Ok((_, warnings)) => messages.extend(warnings.iter().cloned()),
            Err(errors) => messages.extend(errors.iter().cloned()),
        }

        // Collect from taffy_node_mappings
        if let Err(errors) = &*taffy_node_mappings.read() {
            messages.extend(errors.iter().cloned());
        }

        // Collect from svg_elements
        if let Err(errors) = &*svg_elements.read() {
            messages.extend(errors.iter().cloned());
        }

        messages
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
                    taffy_node_mappings_string,
                    svg_elements_string,
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
    taffy_node_mappings_string: Memo<String>,
    svg_elements_string: Memo<String>,
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
            SvgElementsDiv { svg_elements_string }
        }
    }
}

#[component]
fn DispositionStatusMessageDiv(status_messages: Memo<Vec<String>>) -> Element {
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
fn DispositionStatusMessage(status_messages: Memo<Vec<String>>) -> Element {
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
