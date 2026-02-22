use std::{
    fmt::Write,
    hash::{DefaultHasher, Hasher},
};

use dioxus::{
    hooks::{use_memo, use_signal},
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, info, rsx, Element, Props},
    router::navigator,
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
    svg_model::SvgElements,
    taffy_model::{
        taffy::{self, PrintTree},
        DimensionAndLod, TaffyNodeMappings,
    },
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
        editor::{
            EditorDataLists, ProcessesPage, TagsPage, TextPage, ThemeBaseStylesPage,
            ThemeDependenciesStylesPage, ThemeProcessStepStylesPage, ThemeStyleAliasesPage,
            ThemeTagsFocusPage, ThemeTypesStylesPage, ThingDependenciesPage, ThingInteractionsPage,
            ThingsPage,
        },
        IrDiagramDiv, SvgElementsDiv, TabDetails, TabGroup, TaffyNodeMappingsDiv,
    },
    editor_state::{EditorPage, EditorPageOrGroup, EditorState},
    route::Route,
};

/// CSS classes for top-level editor page tabs.
const TAB_CLASS: &str = "\
    cursor-pointer \
    select-none \
    px-3 py-1.5 \
    text-sm \
    font-semibold \
    rounded-t \
    transition-colors \
    duration-150\
";

const TAB_ACTIVE: &str = "text-blue-400 border-b-2 border-blue-400";
const TAB_INACTIVE: &str = "text-gray-400 hover:text-gray-200";

/// CSS classes for theme sub-tabs (smaller, nested).
const SUB_TAB_CLASS: &str = "\
    cursor-pointer \
    select-none \
    px-2 py-1 \
    text-xs \
    font-semibold \
    rounded-t \
    transition-colors \
    duration-150\
";

#[component]
#[allow(clippy::type_complexity)]
pub fn DispositionEditor(editor_state: ReadSignal<EditorState>) -> Element {
    // ── Signals ──────────────────────────────────────────────────────────

    // The InputDiagram being edited, as a read-write signal.
    let mut input_diagram: Signal<InputDiagram<'static>> =
        use_signal(|| editor_state.read().input_diagram.clone());

    // The active editor page.
    let mut active_page: Signal<EditorPage> = use_signal(|| editor_state.read().page.clone());

    // ── Sync: incoming EditorState prop -> local signals ──────────────────

    use_memo(move || {
        let state = editor_state.read();
        if *input_diagram.peek() != state.input_diagram {
            input_diagram.set(state.input_diagram.clone());
        }
        if *active_page.peek() != state.page {
            active_page.set(state.page.clone());
        }
    });

    // ── Sync: local signals -> URL hash (EditorState) ────────────────────

    use_memo(move || {
        let diagram = input_diagram.read().clone();
        let page = active_page.read().clone();

        let current_state = editor_state.peek().clone();
        if current_state.input_diagram != diagram || current_state.page != page {
            navigator().replace(Route::Home {
                editor_state: EditorState {
                    page,
                    input_diagram: diagram,
                },
            });
        }
    });

    // ── InputDiagram as a Memo for read-only consumers ──────────────────

    let input_diagram_memo: Memo<InputDiagram<'static>> =
        use_memo(move || input_diagram.read().clone());

    // ── SVG generation pipeline (unchanged logic) ────────────────────────

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
            let input_diagram_merged = InputDiagramMerger::merge(InputDiagram::base(), &*diagram);
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
                taffy_tree_fmt(&mut buffer, taffy_node_mappings);
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

    // ── Render ───────────────────────────────────────────────────────────

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

            // ── Left column: editor tabs + status + intermediates ────
            div {
                class: "
                    flex-1
                    flex
                    flex-col
                    gap-2
                    min-w-0
                ",

                // ── Top-level tab bar ────────────────────────────────
                EditorTabBar {
                    active_page,
                }

                // ── Active page content ──────────────────────────────
                div {
                    class: "
                        flex-1
                        flex
                        flex-col
                        overflow-y-auto
                        max-h-[70vh]
                        pr-1
                    ",
                    EditorPageContent {
                        active_page,
                        input_diagram,
                    }
                }

                // ── Status messages ──────────────────────────────────
                DispositionStatusMessageDiv { status_messages }

                // ── Intermediate data tabs ───────────────────────────
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

            // ── Right column: SVG preview ────────────────────────────
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

// ===========================================================================
// Editor tab bar
// ===========================================================================

/// Renders the top-level tab bar and, when a Theme page is active, a nested
/// sub-tab bar.
#[component]
fn EditorTabBar(active_page: Signal<EditorPage>) -> Element {
    let current = active_page.read().clone();

    rsx! {
        div {
            class: "flex flex-col",

            // ── Top-level tabs ───────────────────────────────────────
            div {
                class: "
                    flex
                    flex-row
                    flex-wrap
                    gap-1
                    border-b
                    border-gray-700
                    mb-1
                ",

                for entry in EditorPage::TOP_LEVEL.iter() {
                    {
                        let is_active = entry.contains(&current);
                        let css = format!(
                            "{TAB_CLASS} {}",
                            if is_active { TAB_ACTIVE } else { TAB_INACTIVE }
                        );
                        let entry_clone = entry.clone();

                        rsx! {
                            span {
                                key: "{entry.label()}",
                                class: "{css}",
                                onclick: {
                                    let entry = entry_clone.clone();
                                    move |_| {
                                        match &entry {
                                            EditorPageOrGroup::Page(p) => {
                                                active_page.set(p.clone());
                                            }
                                            EditorPageOrGroup::ThemeGroup => {
                                                // If already on a theme page, stay there;
                                                // otherwise default to StyleAliases.
                                                if !active_page.peek().is_theme() {
                                                    active_page.set(EditorPage::ThemeStyleAliases);
                                                }
                                            }
                                        }
                                    }
                                },
                                "{entry.label()}"
                            }
                        }
                    }
                }
            }

            // ── Theme sub-tabs (only visible when a Theme page is active) ──
            if current.is_theme() {
                div {
                    class: "
                        flex
                        flex-row
                        flex-wrap
                        gap-1
                        border-b
                        border-gray-700
                        mb-1
                        pl-2
                    ",

                    for sub in EditorPage::THEME_SUB_PAGES.iter() {
                        {
                            let is_active = current == *sub;
                            let css = format!(
                                "{SUB_TAB_CLASS} {}",
                                if is_active { TAB_ACTIVE } else { TAB_INACTIVE }
                            );
                            let sub_clone = sub.clone();

                            rsx! {
                                span {
                                    key: "{sub.label()}",
                                    class: "{css}",
                                    onclick: {
                                        let sub_page = sub_clone.clone();
                                        move |_| {
                                            active_page.set(sub_page.clone());
                                        }
                                    },
                                    "{sub.label()}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ===========================================================================
// Editor page content dispatcher
// ===========================================================================

/// Renders the content of the currently active editor page.
#[component]
fn EditorPageContent(
    active_page: Signal<EditorPage>,
    input_diagram: Signal<InputDiagram<'static>>,
) -> Element {
    let page = active_page.read().clone();

    match page {
        EditorPage::Things => rsx! { ThingsPage { input_diagram } },
        EditorPage::ThingDependencies => rsx! { ThingDependenciesPage { input_diagram } },
        EditorPage::ThingInteractions => rsx! { ThingInteractionsPage { input_diagram } },
        EditorPage::Processes => rsx! { ProcessesPage { input_diagram } },
        EditorPage::Tags => rsx! { TagsPage { input_diagram } },
        EditorPage::ThemeStyleAliases => rsx! { ThemeStyleAliasesPage { input_diagram } },
        EditorPage::ThemeBaseStyles => rsx! { ThemeBaseStylesPage { input_diagram } },
        EditorPage::ThemeProcessStepStyles => rsx! { ThemeProcessStepStylesPage { input_diagram } },
        EditorPage::ThemeTypesStyles => rsx! { ThemeTypesStylesPage { input_diagram } },
        EditorPage::ThemeDependenciesStyles => {
            rsx! { ThemeDependenciesStylesPage { input_diagram } }
        }
        EditorPage::ThemeTagsFocus => rsx! { ThemeTagsFocusPage { input_diagram } },
        EditorPage::Text => rsx! { TextPage { input_diagram } },
    }
}

// ===========================================================================
// Taffy tree formatting (unchanged)
// ===========================================================================

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

// ===========================================================================
// Status message components (unchanged)
// ===========================================================================

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
