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
        component, dioxus_core, dioxus_elements, dioxus_signals, info, rsx, use_drop, Element,
        Props,
    },
    router::navigator,
    signals::{Memo, ReadSignal, ReadableExt, Signal, WritableExt},
};
use disposition::{
    input_ir_model::IrDiagramAndIssues,
    input_model::{theme::DarkModeCssSelector, InputDiagram},
    ir_model::IrDiagram,
    output_model::DiagramGenerated,
    svg_model::SvgElements,
};
use disposition_input_ir_rt::{
    DiagramGenerator, EdgeAnimationActive, InputToIrDiagramMapper, SvgElementsToSvgMapper,
    TaffyToSvgElementsMapper,
};

use crate::{
    components::{
        editor::EditorDataLists, IrDiagramDiv, SvgElementsDiv, TabDetails, TabGroup,
        TaffyNodeMappingsDiv,
    },
    editor_state::{EditorPage, EditorState},
    hooks::{dark_mode_toggle, use_dark_mode},
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

    // === Light / Dark mode === //
    let dark_mode = use_dark_mode();

    // === Help tooltip visibility === //
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
                var tag = (e.target.tagName || "").toLowerCase();
                var inEditable = (tag === "input" || tag === "textarea" || tag === "select" || e.target.isContentEditable);
                var ctrl = e.ctrlKey || e.metaKey;

                // === l: toggle light/dark mode === //
                // === f: toggle SVG preview expand === //
                // Skip if any modifier is held or focus is in an editable element.
                if (!ctrl && !e.altKey && !e.shiftKey && !inEditable) {{
                    if (e.key === "l") {{
                        e.preventDefault();
                        e.stopPropagation();
                        dioxus.send("dark_mode_toggle");
                        return;
                    }}
                    if (e.key === "f") {{
                        e.preventDefault();
                        e.stopPropagation();
                        dioxus.send("svg_toggle");
                        return;
                    }}
                    if (e.key === "Escape") {{
                        dioxus.send("svg_escape");
                        return;
                    }}
                }}

                // === Shift + ?: toggle help tooltip === //
                // Skip if focus is in an editable element.
                if (e.shiftKey && !ctrl && !e.altKey && !inEditable) {{
                    if (e.key === "?") {{
                        e.preventDefault();
                        e.stopPropagation();
                        dioxus.send("help_toggle");
                        return;
                    }}
                }}

                // === Alt + 0..9: switch top-level tabs === //
                if (e.altKey && !ctrl && !e.shiftKey) {{
                    var digit = e.key.length === 1 ? e.key.charCodeAt(0) - 48 : -1;
                    if (digit >= 0 && digit <= 9) {{
                        e.preventDefault();
                        e.stopPropagation();
                        dioxus.send("tab_" + digit);
                        return;
                    }}
                }}

                // === Ctrl / Meta shortcuts === //
                if (!ctrl) return;

                // Ctrl+Shift+S: open share modal (works even inside inputs).
                if ((e.key === "s" || e.key === "S") && e.shiftKey) {{
                    e.preventDefault();
                    e.stopPropagation();
                    dioxus.send("share");
                    return;
                }}

                // Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y: undo / redo.
                // Skip when focus is in an editable element so the
                // browser's native input undo/redo works.
                if (inEditable) return;

                if (e.key === "z" || e.key === "Z") {{
                    e.preventDefault();
                    e.stopPropagation();
                    dioxus.send(e.shiftKey ? "redo" : "undo");
                    return;
                }}
                if (e.key === "y" || e.key === "Y") {{
                    e.preventDefault();
                    e.stopPropagation();
                    dioxus.send("redo");
                    return;
                }}
            }};
            document.addEventListener("keydown", window["{name}"]);
            "#
        );
        spawn(async move {
            let mut eval = document::eval(&js);
            loop {
                match eval.recv::<String>().await {
                    Ok(msg) if msg == "dark_mode_toggle" => {
                        dark_mode_toggle(dark_mode);
                    }
                    Ok(msg) if msg == "svg_toggle" => {
                        let current = *svg_preview_expanded.read();
                        svg_preview_expanded.set(!current);
                    }
                    Ok(msg) if msg == "svg_escape" => {
                        if *svg_preview_expanded.read() {
                            svg_preview_expanded.set(false);
                        }
                    }
                    Ok(msg) if msg == "help_toggle" => {
                        let current = *show_help.read();
                        show_help.set(!current);
                    }
                    Ok(msg) if msg.starts_with("tab_") => {
                        if let Ok(digit) = msg[4..].parse::<u32>() {
                            let target_index = if digit == 0 {
                                let top_level = EditorPage::top_level_pages();
                                if top_level.is_empty() {
                                    continue;
                                }
                                top_level.len() - 1
                            } else {
                                (digit - 1) as usize
                            };
                            if let Some(page) = EditorPage::default_page(target_index) {
                                if !active_page.peek().same_top_level(&page) {
                                    active_page.set(page);
                                }
                                document::eval(&format!(
                                    "requestAnimationFrame(() => {{\
                                        var el = document.querySelector(\
                                            '[data-top-level-index=\"{target_index}\"]'\
                                        );\
                                        if (el) el.focus();\
                                    }})"
                                ));
                            }
                        }
                    }
                    Ok(msg) if msg == "share" => {
                        show_share_modal.set(true);
                    }
                    Ok(msg) if msg == "undo" => {
                        if let Some(diagram) = history_undo(undo_history) {
                            document::eval(JS_FOCUS_SAVE);
                            input_diagram.set(diagram);
                            document::eval(JS_FOCUS_RESTORE);
                        }
                    }
                    Ok(msg) if msg == "redo" => {
                        if let Some(diagram) = history_redo(undo_history) {
                            document::eval(JS_FOCUS_SAVE);
                            input_diagram.set(diagram);
                            document::eval(JS_FOCUS_RESTORE);
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

    let diagram_generated: Memo<Result<DiagramGenerated, Vec<String>>> = use_memo(move || {
        let diagram = input_diagram.read();
        if diagram.things.is_empty()
            && diagram.thing_dependencies.is_empty()
            && diagram.thing_interactions.is_empty()
            && diagram.processes.is_empty()
        {
            return Err(vec![String::from("ℹ️ Add things to get started")]);
        }

        let diagram_generated =
            DiagramGenerator::generate(&diagram, EdgeAnimationActive::OnProcessStepFocus)
                .map_err(|error| vec![format!("⚠️ Error generating diagram: {error}")])?;

        info!(
            "`DiagramGenerator::generate` took: merge {} ms, ir {} ms, taffy {} ms, svg_elements {} ms, svg {} ms.",
            diagram_generated.input_diagram_merged_merge_duration.as_millis(),
            diagram_generated.ir_diagram_map_duration.as_millis(),
            diagram_generated.taffy_node_mappings_build_duration.as_millis(),
            diagram_generated.svg_elements_map_duration.as_millis(),
            diagram_generated.svg_map_duration.as_millis(),
        );

        Ok(diagram_generated)
    });

    let ir_diagram_string: Memo<String> = use_memo(move || {
        match &*diagram_generated.read() {
            Ok(diagram_generated) => {
                let mut buffer = String::new();
                match serde_saphyr::to_fmt_writer(&mut buffer, &diagram_generated.ir_diagram) {
                    Ok(()) => buffer,
                    Err(error) => format!("⚠️ Error serializing IR diagram: {}", error),
                }
            }
            Err(_) => String::new(),
        }
    });

    let taffy_node_mappings_string: Memo<String> = use_memo(move || {
        match &*diagram_generated.read() {
            Ok(diagram_generated) => {
                let mut buffer = String::new();
                TaffyTreeFmt::fmt(&mut buffer, &diagram_generated.taffy_node_mappings);
                buffer
            }
            Err(_) => String::new(),
        }
    });

    let svg_elements_string: Memo<String> = use_memo(move || {
        match &*diagram_generated.read() {
            Ok(diagram_generated) => {
                let mut buffer = String::new();
                match serde_saphyr::to_fmt_writer(&mut buffer, &diagram_generated.svg_elements) {
                    Ok(()) => buffer,
                    Err(error) => format!("⚠️ Error serializing SVG elements: {}", error),
                }
            }
            Err(_) => String::new(),
        }
    });

    let svg: Memo<String> = use_memo(move || match &*diagram_generated.read() {
        Ok(diagram_generated) => diagram_generated.svg.clone(),
        Err(_) => String::new(),
    });

    // === Display SVG pipeline (forces `RootDarkClass`) === //
    //
    // The intermediate data and `svg` above are generated from the user's
    // `InputDiagram`, whose dark-mode CSS selector defaults to `MediaQuery` --
    // the sensible default to generate / copy.
    //
    // For the *displayed* SVG we instead force
    // `DarkModeCssSelector::RootDarkClass`, so the diagram's dark mode tracks
    // the site's `dark` class toggle. This lets the user preview how the
    // diagram looks in light / dark mode, matching the rest of the site.
    //
    // Only the dark-mode CSS block (in `IrDiagram::css`) differs between the
    // two selectors; node layout is identical, so the `taffy_node_mappings`
    // computed above are reused rather than recomputed.

    let ir_diagram_display: Memo<Option<IrDiagram<'static>>> = use_memo(move || {
        let diagram_generated = diagram_generated.read();
        let Ok(diagram_generated) = &*diagram_generated else {
            return None;
        };

        // Reuse the merged input diagram from the generation pipeline, only
        // overriding the dark-mode selector so the displayed diagram tracks the
        // site's `dark` class toggle.
        let mut input_diagram_merged = diagram_generated.input_diagram_merged.clone();
        input_diagram_merged.theme_default.dark_mode_config.selector =
            DarkModeCssSelector::RootDarkClass;

        let IrDiagramAndIssues { diagram, .. } = InputToIrDiagramMapper::map(&input_diagram_merged);
        Some(diagram)
    });

    let svg_elements_display: Memo<Option<SvgElements>> = use_memo(move || {
        let ir_diagram_display = &*ir_diagram_display.read();
        let diagram_generated = diagram_generated.read();
        match (ir_diagram_display, &*diagram_generated) {
            (Some(ir_diagram), Ok(diagram_generated)) => Some(TaffyToSvgElementsMapper::map(
                ir_diagram,
                &diagram_generated.taffy_node_mappings,
                EdgeAnimationActive::OnProcessStepFocus,
            )),
            _ => None,
        }
    });

    let svg_display: Memo<String> = use_memo(move || {
        // Embed the user's original `InputDiagram` (with its `MediaQuery`
        // default) as the `<source>`, so copying / sharing yields the sensible
        // default, while the rendered styles use `RootDarkClass`.
        let input_diagram = input_diagram.read();
        let svg_elements_display = &*svg_elements_display.read();
        match svg_elements_display {
            Some(svg_elements) => {
                SvgElementsToSvgMapper::map_with_input(&input_diagram, svg_elements)
            }
            None => String::new(),
        }
    });

    // Collect all status messages.
    let status_messages: Memo<Vec<String>> = use_memo(move || {
        let mut messages = Vec::new();

        match &*diagram_generated.read() {
            Ok(diagram_generated) => {
                if !diagram_generated.ir_diagram_issues.is_empty() {
                    messages.push(String::from(
                        "⚠️ Issues mapping input diagram to IR diagram",
                    ));
                    messages.extend(
                        diagram_generated
                            .ir_diagram_issues
                            .iter()
                            .map(|issue| issue.to_string()),
                    );
                }
            }
            Err(errors) => messages.extend(errors.iter().cloned()),
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

            // All global keyboard shortcuts are handled by the JS-level
            // `document` keydown listener registered above. This avoids
            // Dioxus `onkeydown` intercepting Ctrl+Z/Y inside `<input>`
            // elements, which would prevent the browser's native
            // input undo/redo from working.

            // === Left column: editor tabs + status + intermediates === //
            div {
                class: "
                    flex
                    flex-col
                    gap-2
                    min-w-sm
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
                        class: "flex-1",
                        EditorTabBar {
                            active_page,
                        }
                    }

                    div {
                        class: "\
                            flex-shrink \
                            flex \
                            flex-col \
                            gap-2 \
                            justify-between\
                        ",

                        ExampleDiagramSelect {
                            input_diagram,
                        }

                        div {
                            class: "\
                                flex \
                                gap-2 \
                                items-center \
                                justify-end\
                            ",
                            UndoRedoToolbar {
                                input_diagram,
                                undo_history,
                            }

                            HelpTooltip { show_help }
                        }
                    }
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
                                svg_display,
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
