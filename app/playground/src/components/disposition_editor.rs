//! Disposition editor component.
//!
//! The main editor component wiring together the tab bar, page content,
//! status messages, and SVG preview.

mod disposition_status_message_div;
mod editor_page_content;
mod editor_tab_bar;
mod taffy_tree_fmt;

use dioxus::{
    hooks::{use_memo, use_signal},
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, info, rsx, Element, Props},
    router::navigator,
    signals::{Memo, ReadSignal, ReadableExt, Signal, WritableExt},
};
use disposition::{
    input_ir_model::IrDiagramAndIssues,
    input_model::InputDiagram,
    ir_model::IrDiagram,
    svg_model::SvgElements,
    taffy_model::{DimensionAndLod, TaffyNodeMappings},
};
use disposition_input_ir_rt::{
    EdgeAnimationActive, InputDiagramMerger, InputToIrDiagramMapper, IrToTaffyBuilder,
    SvgElementsToSvgMapper, TaffyToSvgElementsMapper,
};

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

use crate::{
    components::{
        editor::EditorDataLists, IrDiagramDiv, SvgElementsDiv, TabDetails, TabGroup,
        TaffyNodeMappingsDiv,
    },
    editor_state::{EditorPage, EditorState, ThingsPageUiState},
    route::Route,
};

use self::{
    disposition_status_message_div::DispositionStatusMessageDiv,
    editor_page_content::EditorPageContent, editor_tab_bar::EditorTabBar,
    taffy_tree_fmt::TaffyTreeFmt,
};

#[component]
#[allow(clippy::type_complexity)]
pub fn DispositionEditor(editor_state: ReadSignal<EditorState>) -> Element {
    // === Signals === //

    // The InputDiagram being edited, as a read-write signal.
    let mut input_diagram: Signal<InputDiagram<'static>> =
        use_signal(|| editor_state.read().input_diagram.clone());

    // The active editor page.
    let mut active_page: Signal<EditorPage> = use_signal(|| editor_state.read().page.clone());

    // UI state for the Things page (collapsed sections).
    let mut things_ui_state: Signal<ThingsPageUiState> =
        use_signal(|| editor_state.read().things_ui.clone());

    // === Sync: incoming EditorState prop -> local signals === //

    use_memo(move || {
        let state = editor_state.read();
        if *input_diagram.peek() != state.input_diagram {
            input_diagram.set(state.input_diagram.clone());
        }
        if *active_page.peek() != state.page {
            active_page.set(state.page.clone());
        }
        if *things_ui_state.peek() != state.things_ui {
            things_ui_state.set(state.things_ui.clone());
        }
    });

    // === Sync: local signals -> URL hash (EditorState) === //

    use_memo(move || {
        let diagram = input_diagram.read().clone();
        let page = active_page.read().clone();
        let things_ui = things_ui_state.read().clone();

        let current_state = editor_state.peek().clone();
        if current_state.input_diagram != diagram
            || current_state.page != page
            || current_state.things_ui != things_ui
        {
            navigator().replace(Route::Home {
                editor_state: EditorState {
                    page,
                    input_diagram: diagram,
                    things_ui,
                },
            });
        }
    });

    // === InputDiagram as a Memo for read-only consumers === //

    let input_diagram_memo: Memo<InputDiagram<'static>> =
        use_memo(move || input_diagram.read().clone());

    // === SVG generation pipeline === //

    let ir_diagram: Memo<Result<(IrDiagram<'static>, Vec<String>), Vec<String>>> =
        use_memo(move || {
            let diagram = input_diagram.read();
            if diagram.things.is_empty()
                && diagram.thing_dependencies.is_empty()
                && diagram.thing_interactions.is_empty()
            {
                return Err(vec![String::from("ℹ️ Add things to get started")]);
            }

            let input_diagram_merge_start = Instant::now();
            let input_diagram_merged = InputDiagramMerger::merge(InputDiagram::base(), &diagram);
            let input_diagram_merge_duration_ms = Instant::now()
                .duration_since(input_diagram_merge_start)
                .as_millis();
            info!("`InputDiagramMerger::merge` took {input_diagram_merge_duration_ms} ms.");

            let input_to_ir_map_start = Instant::now();
            let input_diagram_and_issues = InputToIrDiagramMapper::map(&input_diagram_merged);
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
        });

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
                Err(_) => Err(vec![]),
            }
        },
    );

    let taffy_node_mappings_string: Memo<String> = use_memo(move || {
        let taffy_node_mappings = &*taffy_node_mappings.read();
        match taffy_node_mappings {
            Ok(taffy_node_mappings) => {
                let mut buffer = String::new();
                TaffyTreeFmt::fmt(&mut buffer, taffy_node_mappings);
                buffer
            }
            Err(_) => String::new(),
        }
    });

    let svg_elements: Memo<Result<SvgElements, Vec<String>>> = use_memo(move || {
        let ir_diagram = &*ir_diagram.read();
        let taffy_node_mappings = &*taffy_node_mappings.read();

        match (ir_diagram, taffy_node_mappings) {
            (Ok((ir_diagram, _)), Ok(taffy_node_mappings)) => {
                let svg_elements_map_start = Instant::now();
                let svg_elements = TaffyToSvgElementsMapper::map(
                    ir_diagram,
                    taffy_node_mappings,
                    EdgeAnimationActive::OnProcessStepFocus,
                );
                let svg_elements_map_duration_ms = Instant::now()
                    .duration_since(svg_elements_map_start)
                    .as_millis();
                info!("`TaffyToSvgElementsMapper::map` took {svg_elements_map_duration_ms} ms.");
                Ok(svg_elements)
            }
            _ => Err(vec![]),
        }
    });

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

    // Collect all status messages.
    let status_messages: Memo<Vec<String>> = use_memo(move || {
        let mut messages = Vec::new();

        match &*ir_diagram.read() {
            Ok((_, warnings)) => messages.extend(warnings.iter().cloned()),
            Err(errors) => messages.extend(errors.iter().cloned()),
        }
        if let Err(errors) = &*taffy_node_mappings.read() {
            messages.extend(errors.iter().cloned());
        }
        if let Err(errors) = &*svg_elements.read() {
            messages.extend(errors.iter().cloned());
        }
        messages
    });

    // === Render === //

    rsx! {
        // Hidden datalist elements available to all pages.
        EditorDataLists { input_diagram: input_diagram_memo }

        div {
            id: "disposition_editor",
            class: "
                flex
                flex-col
                lg:flex-row
                gap-2
            ",

            // === Left column: editor tabs + status + intermediates === //
            div {
                class: "
                    flex-1
                    flex
                    flex-col
                    gap-2
                    min-w-0
                ",

                EditorTabBar {
                    active_page,
                }

                div {
                    class: "
                        flex-1
                        flex
                        flex-col
                        overflow-y-auto
                        max-h-[70vh]
                        pr-1
                    ",
                    tabindex: "-1",
                    EditorPageContent {
                        active_page,
                        input_diagram,
                        things_ui_state,
                    }
                }

                DispositionStatusMessageDiv { status_messages }

                TabGroup {
                    group_name: "intermediate_tabs",
                    default_checked: 0usize,
                    tabs: vec![
                        TabDetails {
                            label: String::from("IR Diagram"),
                            content: rsx! { IrDiagramDiv { ir_diagram_string } },
                        },
                        TabDetails {
                            label: String::from("Taffy Node Mappings"),
                            content: rsx! { TaffyNodeMappingsDiv { taffy_node_mappings_string } },
                        },
                        TabDetails {
                            label: String::from("SVG Elements"),
                            content: rsx! { SvgElementsDiv { svg_elements_string } },
                        },
                    ],
                }
            }

            // === Right column: SVG preview === //
            object {
                class: "
                    flex-1
                ",
                r#type: "image/svg+xml",
                data: format!("data:image/svg+xml,{}", urlencoding::encode(svg().as_str())),
            }
        }
    }
}
