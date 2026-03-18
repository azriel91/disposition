//! Disposition editor component.
//!
//! The main editor component wiring together the tab bar, page content,
//! status messages, and SVG preview.

mod copy_button;
mod disposition_status_message_div;
mod editor_page_content;
mod editor_tab_bar;
mod example_diagram_select;
mod focus_restore;
mod help_tooltip;
mod share_button;
mod share_modal;
mod svg_preview;
mod taffy_tree_fmt;
mod undo_redo_toolbar;

use dioxus::{
    core::spawn,
    document,
    hooks::{use_context_provider, use_effect, use_memo, use_signal},
    prelude::{
        component, dioxus_core, dioxus_elements, dioxus_signals, info, rsx, use_drop, Element, Key,
        ModifiersInteraction, Props,
    },
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
    editor_state::{EditorPage, EditorState},
    route::Route,
    undo_history::{history_push, history_redo, history_undo, UndoHistory},
};

pub(crate) use self::{
    copy_button::CopyButton,
    disposition_status_message_div::DispositionStatusMessageDiv,
    editor_page_content::EditorPageContent,
    editor_tab_bar::EditorTabBar,
    example_diagram_select::ExampleDiagramSelect,
    focus_restore::{JS_FOCUS_RESTORE, JS_FOCUS_SAVE},
    help_tooltip::HelpTooltip,
    share_button::ShareButton,
    share_modal::ShareModal,
    svg_preview::SvgPreview,
    taffy_tree_fmt::TaffyTreeFmt,
    undo_redo_toolbar::UndoRedoToolbar,
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

    // Whether the SVG preview is expanded to fill the entire viewport.
    let mut svg_preview_expanded: Signal<bool> =
        use_signal(|| editor_state.read().svg_preview_expanded);

    // === Focus field: expand + focus a specific field on page load === //

    // Capture the initial `focus_field` from the URL. This signal is provided
    // via context so that card components can check whether they should start
    // expanded. It is consumed once (on mount) and then cleared.
    let focus_field: Signal<Option<String>> =
        use_signal(|| editor_state.read().focus_field.clone());
    use_context_provider(|| focus_field);

    // The base diagram for comparison. Theme editor components read this
    // via `use_context` to show which values are from the base vs. user
    // overrides.
    let base_diagram: Memo<InputDiagram<'static>> = use_memo(InputDiagram::base);
    use_context_provider(|| base_diagram);

    // After the first render, run a JS snippet that focuses the target
    // element and then clear `focus_field` from the URL so subsequent
    // edits don't re-trigger the focus.
    use_effect(move || {
        let field_value = focus_field.read().clone();
        if let Some(ref field_id) = field_value {
            let selector = format!("[data-input-diagram-field=\"{field_id}\"]");
            let js = format!(
                "requestAnimationFrame(() => {{\
                    var el = document.querySelector('{selector}');\
                    if (el) el.focus();\
                }})"
            );
            document::eval(&js);

            // Strip `focus_field` from the URL so it doesn't persist.
            let diagram = input_diagram.peek().clone();
            let page = active_page.peek().clone();
            navigator().replace(Route::Home {
                editor_state: EditorState {
                    page,
                    focus_field: None,
                    svg_preview_expanded: *svg_preview_expanded.peek(),
                    input_diagram: diagram,
                },
            });
        }
    });

    // === Last focused field: tracked for the share modal === //

    // Stores the `data-input-diagram-field` attribute value of the most
    // recently focused editor field. Updated via a `focusin` listener on
    // the editor root so it is always current, even after the user clicks
    // the Share button (which moves focus away from the field).
    let mut last_focused_field: Signal<Option<String>> = use_signal(|| None);

    // Share modal visibility.
    let mut show_share_modal: Signal<bool> = use_signal(|| false);

    // The editor state as a writable signal, kept in sync with
    // `input_diagram` and `active_page` for the share modal.
    let mut editor_state_for_share: Signal<EditorState> = use_signal(|| EditorState {
        page: active_page.peek().clone(),
        focus_field: None,
        svg_preview_expanded: *svg_preview_expanded.peek(),
        input_diagram: input_diagram.peek().clone(),
    });
    use_memo(move || {
        let diagram = input_diagram.read().clone();
        let page = active_page.read().clone();
        editor_state_for_share.write().page = page;
        editor_state_for_share.write().input_diagram = diagram;
    });

    // === Undo history === //

    let undo_history: Signal<UndoHistory> =
        use_signal(|| UndoHistory::new(input_diagram.peek().clone()));

    // Watch `input_diagram` for changes and push snapshots to the history.
    // The `UndoHistory::push` method internally handles the `skip_next_push`
    // flag so that undo/redo-triggered writes don't create new entries.
    use_memo(move || {
        let diagram = input_diagram.read().clone();
        history_push(undo_history, diagram);
    });

    // Help tooltip visibility.
    let mut show_help: Signal<bool> = use_signal(|| false);

    // === Global JS keydown listener for `f` / `Escape` expand toggle === //
    //
    // We use a JS-level listener so we can inspect the focused element and
    // skip the shortcut when the user is typing in an input / textarea /
    // select / contentEditable field. A Rust `onkeydown` handler cannot
    // synchronously query the DOM target.
    //
    // The listener is registered once on mount and removed on drop. It sends
    // `"toggle"` or `"escape"` strings back to Rust via `dioxus.send()`.

    static LISTENER_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let listener_name = use_signal(|| {
        let id = LISTENER_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        format!("__disposition_expand_listener_{id}")
    });

    // Register the listener once (no reactive reads inside this effect).
    use_effect(move || {
        let name = listener_name.read().clone();
        let js = format!(
            r#"
            if (window["{name}"]) {{
                document.removeEventListener("keydown", window["{name}"]);
            }}
            window["{name}"] = function(e) {{
                // Skip if modifier keys are held.
                if (e.ctrlKey || e.metaKey || e.altKey || e.shiftKey) return;
                // Skip if focus is in an editable element.
                var tag = (e.target.tagName || "").toLowerCase();
                if (tag === "input" || tag === "textarea" || tag === "select") return;
                if (e.target.isContentEditable) return;

                if (e.key === "f") {{
                    e.preventDefault();
                    e.stopPropagation();
                    dioxus.send("toggle");
                }} else if (e.key === "Escape") {{
                    dioxus.send("escape");
                }}
            }};
            document.addEventListener("keydown", window["{name}"]);
            "#
        );
        spawn(async move {
            let mut eval = document::eval(&js);
            loop {
                match eval.recv::<String>().await {
                    Ok(msg) if msg == "toggle" => {
                        let current = *svg_preview_expanded.read();
                        svg_preview_expanded.set(!current);
                    }
                    Ok(msg) if msg == "escape" => {
                        if *svg_preview_expanded.read() {
                            svg_preview_expanded.set(false);
                        }
                    }
                    _ => break,
                }
            }
        });
    });

    // Clean up the global listener when the component is unmounted.
    use_drop(move || {
        let name = listener_name.read().clone();
        document::eval(&format!(
            r#"
            if (window["{name}"]) {{
                document.removeEventListener("keydown", window["{name}"]);
                delete window["{name}"];
            }}
            "#
        ));
    });

    // JavaScript snippet that captures the `data-input-diagram-field`
    // attribute of the focused element and sends it back to Rust via
    // `dioxus.send()`. Used in the `onfocusin` handler below.
    const JS_CAPTURE_FOCUSED_FIELD: &str = "\
        var el = document.activeElement;\
        var field = el ? el.closest('[data-input-diagram-field]') : null;\
        dioxus.send(field ? field.getAttribute('data-input-diagram-field') : '');";

    // === Sync: incoming EditorState prop -> local signals === //

    use_memo(move || {
        let state = editor_state.read();
        if *input_diagram.peek() != state.input_diagram {
            input_diagram.set(state.input_diagram.clone());
        }
        if *active_page.peek() != state.page {
            active_page.set(state.page.clone());
        }
        if *svg_preview_expanded.peek() != state.svg_preview_expanded {
            svg_preview_expanded.set(state.svg_preview_expanded);
        }
    });

    // === Sync: local signals -> URL hash (EditorState) === //

    use_memo(move || {
        let diagram = input_diagram.read().clone();
        let page = active_page.read().clone();
        let expanded = *svg_preview_expanded.read();
        let current_state = editor_state.peek().clone();
        if current_state.input_diagram != diagram
            || current_state.page != page
            || current_state.svg_preview_expanded != expanded
        {
            navigator().replace(Route::Home {
                editor_state: EditorState {
                    page,
                    focus_field: None,
                    svg_preview_expanded: expanded,
                    input_diagram: diagram,
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
            tabindex: "-1",
            class: "
                flex
                flex-col
                lg:flex-row
                gap-2
                [&>*]:flex-1
            ",
            // Track the last focused `data-input-diagram-field` element
            // so the share modal can include it in the URL.
            onfocusin: move |_| {
                spawn(async move {
                    let result = document::eval(JS_CAPTURE_FOCUSED_FIELD)
                        .recv::<String>()
                        .await
                        .ok()
                        .unwrap_or_default();
                    if !result.is_empty() {
                        last_focused_field.set(Some(result));
                    }
                });
            },

            // Global keyboard shortcuts:
            //
            // - f = toggle SVG preview expand.
            // - Escape = collapse SVG preview (when expanded).
            // - ctrl + z = undo, ctrl + shift + z / ctrl + y = redo.
            // - alt + 1..9 = switch to top-level tab N.
            // - alt + 0 = switch to the last top-level tab.
            // - shift + ? = show help tooltip (keyboard shortcuts).
            // - ctrl + shift + s = open share modal.
            //
            // Meta (Cmd on macOS) is also supported for undo/redo and share.
            onkeydown: move |evt| {
                if evt.modifiers().shift() && let Key::Character(ref c) = evt.key() && c == "?" {
                    let show_help_current = *show_help.read();
                    show_help.set(!show_help_current);
                }

                // === alt + 0..9: switch top-level tabs === //
                if evt.modifiers().alt()
                    && let Key::Character(ref c) = evt.key()
                        && let Some(digit) = c.chars().next().and_then(|ch| ch.to_digit(10)) {
                            let target_index = if digit == 0 {
                                // Alt+0 -> last tab.
                                let top_level = EditorPage::top_level_pages();
                                if top_level.is_empty() {
                                    return;
                                }
                                top_level.len() - 1
                            } else {
                                // Alt+1..9 -> tab at index digit-1.
                                (digit - 1) as usize
                            };
                            if let Some(page) = EditorPage::default_page(target_index) {
                                evt.prevent_default();
                                evt.stop_propagation();
                                // For grouped tabs (Thing/Theme), preserve
                                // the current sub-tab if already in the
                                // same group.
                                if !active_page.peek().same_top_level(&page) {
                                    active_page.set(page);
                                }
                                // Focus the tab span so subsequent
                                // hotkeys are captured by the editor.
                                let js = format!(
                                    "requestAnimationFrame(() => {{\
                                        var el = document.querySelector(\
                                            '[data-top-level-index=\"{target_index}\"]'\
                                        );\
                                        if (el) el.focus();\
                                    }})"
                                );
                                document::eval(&js);
                            }
                            return;
                        }

                // === ctrl / meta shortcuts: undo / redo === //
                let ctrl_or_meta = evt.modifiers().ctrl() || evt.modifiers().meta();
                if !ctrl_or_meta {
                    return;
                }

                match evt.key() {
                    Key::Character(ref c) if c.eq_ignore_ascii_case("k") => {
                        evt.prevent_default();
                        evt.stop_propagation();
                        show_share_modal.set(true);
                    }
                    Key::Character(ref c) if c.eq_ignore_ascii_case("z") => {
                        evt.prevent_default();
                        evt.stop_propagation();
                        if evt.modifiers().shift() {
                            // Ctrl+Shift+Z -> redo
                            if let Some(diagram) = history_redo(undo_history) {
                                document::eval(JS_FOCUS_SAVE);
                                input_diagram.set(diagram);
                                document::eval(JS_FOCUS_RESTORE);
                            }
                        } else {
                            // Ctrl+Z -> undo
                            if let Some(diagram) = history_undo(undo_history) {
                                document::eval(JS_FOCUS_SAVE);
                                input_diagram.set(diagram);
                                document::eval(JS_FOCUS_RESTORE);
                            }
                        }
                    }
                    Key::Character(ref c) if c.eq_ignore_ascii_case("y") => {
                        evt.prevent_default();
                        evt.stop_propagation();
                        // Ctrl+Y -> redo
                        if let Some(diagram) = history_redo(undo_history) {
                            document::eval(JS_FOCUS_SAVE);
                            input_diagram.set(diagram);
                            document::eval(JS_FOCUS_RESTORE);
                        }
                    }
                    _ => {}
                }
            },

            // === Left column: editor tabs + status + intermediates === //
            div {
                class: "
                    flex
                    flex-col
                    gap-2
                    min-w-0
                ",

                // === Tab bar + undo/redo toolbar === //
                div {
                    class: "
                        flex
                        flex-row
                        items-start
                        gap-2
                    ",

                    div {
                        class: "flex-1 min-w-0",
                        EditorTabBar {
                            active_page,
                        }
                    }

                    ExampleDiagramSelect {
                        input_diagram,
                    }

                    UndoRedoToolbar {
                        input_diagram,
                        undo_history,
                    }

                    HelpTooltip { show_help }
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
                    }
                }

                DispositionStatusMessageDiv { status_messages }
            }

            // === Right column: Intermediate transformations and SVG preview === //

            TabGroup {
                group_name: "intermediate_tabs",
                default_checked: 0usize,
                tabs: vec![
                    TabDetails {
                        label: String::from("SVG Preview"),
                        content: rsx! {
                            SvgPreview {
                                svg,
                                show_share_modal,
                                svg_preview_expanded,
                            }
                        },
                    },
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

        // === Share modal === //
        ShareModal {
            show: show_share_modal,
            editor_state: editor_state_for_share,
            last_focused_field,
            svg_preview_expanded,
        }
    }
}
