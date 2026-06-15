use std::path::{Path, PathBuf};

use clap::{Parser, ValueEnum};
use disposition::{
    input_model::{DiagramFocus, InputDiagram},
    ir_model::entity::EntityTailwindClasses,
    model_common::theme::Css,
    output_model::DiagramGenerated,
    taffy_model::TaffyTreeFmt,
};
use disposition_input_ir_rt::{
    DiagramGenerateError, DiagramGenerator, EdgeAnimationActive, SvgElementsToSvgMapper,
};
use thiserror::Error;

/// Generates diagram artifacts from an input YAML diagram.
///
/// By default, writes the following files to the output directory:
///
/// * `ir_diagram.yaml`: the intermediate representation diagram
/// * `taffy_tree.txt`: the taffy layout tree
/// * `svg_elements.yaml`: the SVG elements
/// * `diagram.svg`: the final SVG
///
/// Use `--data` to restrict output to a single intermediate stage, and
/// `--stdout` to write that stage straight to stdout -- useful when debugging
/// diagram generation without needing to write files.
///
/// Use `--diagram-per-interaction` to generate one diagram per focus state (no
/// focus, each process, each process step, each tag), with the focus baked in
/// statically. Each output file is then prefixed with the focus ID (`none` for
/// the no-focus diagram), except `taffy_tree.txt` which is shared by all. When
/// writing to stdout, each diagram is preceded by a `<!-- focus: ID -->`
/// comment header.
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Path to the input diagram YAML file.
    input: PathBuf,
    /// Directory to write output files to.
    ///
    /// Required unless `--stdout` is specified.
    output: Option<PathBuf>,
    /// Only output values relevant to the structure of the diagram, without any
    /// styles or colors.
    #[arg(long)]
    structure_only: bool,
    /// Which intermediate diagram data to output.
    ///
    /// When unspecified, all stages are written to the output directory. When
    /// `--stdout` is specified without this, defaults to `svg`.
    #[arg(long, value_enum)]
    data: Option<Data>,
    /// Output the selected `--data` to stdout instead of (or in addition to)
    /// writing files.
    #[arg(long)]
    stdout: bool,
    /// Generate one diagram per process step / tag (and per process, plus a
    /// no-focus diagram), instead of a single interactive diagram.
    #[arg(long)]
    diagram_per_interaction: bool,
}

/// An intermediate diagram transformation stage that can be output.
///
/// The variants are ordered by the diagram generation pipeline, so later stages
/// require all earlier stages to be computed first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Data {
    /// The intermediate representation diagram.
    IrDiagram,
    /// The taffy layout tree.
    TaffyTree,
    /// The SVG elements.
    SvgElements,
    /// The final SVG.
    Svg,
}

#[derive(Debug, Error)]
enum CliError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("yaml deserialize: {0}")]
    YamlDeserialize(#[from] serde_saphyr::Error),
    #[error("yaml serialize: {0}")]
    YamlSerialize(#[from] serde_saphyr::ser::Error),
    #[error("generate: {0}")]
    Generate(#[from] DiagramGenerateError),
    #[error("no output specified: provide an output directory or `--stdout`")]
    NoOutput,
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), CliError> {
    let Args {
        input,
        output,
        structure_only,
        data,
        stdout,
        diagram_per_interaction,
    } = Args::parse();

    if output.is_none() && !stdout {
        return Err(CliError::NoOutput);
    }

    // The stage to output. When writing to stdout without an explicit `--data`,
    // default to the final SVG. When `data` is `None`, all stages are output.
    let data_selected = if stdout {
        Some(data.unwrap_or(Data::Svg))
    } else {
        data
    };
    let data_is_selected =
        |data: Data| data_selected.is_none_or(|data_selected| data_selected == data);

    let contents = tokio::fs::read_to_string(&input).await?;
    let input_diagram: InputDiagram<'static> = serde_saphyr::from_str(&contents)?;

    if let Some(output) = output.as_deref() {
        tokio::fs::create_dir_all(output).await?;
    }
    let output = output.as_deref();

    if diagram_per_interaction {
        let diagrams_focus_generated = DiagramGenerator::generate_per_process_step_or_tag(
            &input_diagram,
            EdgeAnimationActive::OnProcessStepFocus,
        )?;

        // The IR mapping issues are identical across every focus, so report them
        // once.
        if let Some(diagram_focus_generated) = diagrams_focus_generated.first() {
            issues_report(&diagram_focus_generated.diagram_generated);
        }

        // The taffy layout is focus-independent, so it is shared by every
        // diagram and emitted once, unprefixed.
        if data_is_selected(Data::TaffyTree)
            && let Some(diagram_focus_generated) = diagrams_focus_generated.first()
        {
            taffy_tree_emit(
                &diagram_focus_generated.diagram_generated,
                output,
                stdout,
                Some("<!-- taffy_tree (common) -->"),
            )
            .await?;
        }

        for diagram_focus_generated in &diagrams_focus_generated {
            let file_prefix = focus_id(&diagram_focus_generated.focus);
            let stdout_header = format!("<!-- focus: {file_prefix} -->");
            diagram_stages_emit(
                &diagram_focus_generated.diagram_generated,
                &input_diagram,
                structure_only,
                output,
                stdout,
                data_selected,
                Some(&file_prefix),
                Some(&stdout_header),
            )
            .await?;
        }
    } else {
        let diagram_generated =
            DiagramGenerator::generate(&input_diagram, EdgeAnimationActive::OnProcessStepFocus)?;

        issues_report(&diagram_generated);

        if data_is_selected(Data::TaffyTree) {
            taffy_tree_emit(&diagram_generated, output, stdout, None).await?;
        }

        diagram_stages_emit(
            &diagram_generated,
            &input_diagram,
            structure_only,
            output,
            stdout,
            data_selected,
            None,
            None,
        )
        .await?;
    }

    Ok(())
}

/// Reports any input-to-IR mapping issues to stderr.
fn issues_report(diagram_generated: &DiagramGenerated) {
    if !diagram_generated.ir_diagram_issues.is_empty() {
        eprintln!("Issues mapping input to IR diagram:");
        for issue in &diagram_generated.ir_diagram_issues {
            eprintln!("  {issue}");
        }
    }
}

/// Returns the file-name / stdout-header prefix for a [`DiagramFocus`].
///
/// Uses the focused entity's ID, or `none` for the no-focus diagram.
fn focus_id(focus: &DiagramFocus<'_>) -> String {
    match focus {
        DiagramFocus::None => "none".to_string(),
        DiagramFocus::Process(process_id) => process_id.as_str().to_string(),
        DiagramFocus::ProcessStep {
            process_step_id, ..
        } => process_step_id.as_str().to_string(),
        DiagramFocus::Tag(tag_id) => tag_id.as_str().to_string(),
    }
}

/// Returns the output file name for a stage, applying `file_prefix` when set.
///
/// e.g. `("proc_one_step_build", "ir_diagram.yaml")` ->
/// `proc_one_step_build_ir_diagram.yaml`.
fn file_name(file_prefix: Option<&str>, base: &str) -> String {
    match file_prefix {
        Some(file_prefix) => format!("{file_prefix}_{base}"),
        None => base.to_string(),
    }
}

/// Emits the taffy layout tree, optionally preceded by a stdout header.
async fn taffy_tree_emit(
    diagram_generated: &DiagramGenerated,
    output: Option<&Path>,
    stdout: bool,
    stdout_header: Option<&str>,
) -> Result<(), CliError> {
    let mut taffy_tree = String::new();
    TaffyTreeFmt::fmt(&mut taffy_tree, &diagram_generated.taffy_node_mappings);
    if stdout && let Some(stdout_header) = stdout_header {
        println!("{stdout_header}");
    }
    data_emit(output, stdout, "taffy_tree.txt", &taffy_tree).await?;
    Ok(())
}

/// Emits the `ir_diagram`, `svg_elements`, and `svg` stages for one diagram.
///
/// The taffy tree is emitted separately via [`taffy_tree_emit`], because it is
/// identical across all per-interaction diagrams.
///
/// * `file_prefix`: prefix applied to each output file name, e.g.
///   `Some("proc_one_step_build")`; `None` writes the bare stage names.
/// * `stdout_header`: a header line printed to stdout once before this
///   diagram's stage output, e.g. `Some("<!-- focus: proc_one_step_build -->")`.
#[allow(clippy::too_many_arguments)]
async fn diagram_stages_emit(
    diagram_generated: &DiagramGenerated,
    input_diagram: &InputDiagram<'static>,
    structure_only: bool,
    output: Option<&Path>,
    stdout: bool,
    data_selected: Option<Data>,
    file_prefix: Option<&str>,
    stdout_header: Option<&str>,
) -> Result<(), CliError> {
    let data_is_selected =
        |data: Data| data_selected.is_none_or(|data_selected| data_selected == data);

    // Print the stdout header once before this diagram's stage output, but only
    // when at least one of its stages will actually be written to stdout.
    if stdout
        && let Some(stdout_header) = stdout_header
        && (data_is_selected(Data::IrDiagram)
            || data_is_selected(Data::SvgElements)
            || data_is_selected(Data::Svg))
    {
        println!("{stdout_header}");
    }

    // === IR diagram === //
    if data_is_selected(Data::IrDiagram) {
        let mut ir_yaml = String::new();
        if structure_only {
            // `tailwind_classes` / `css` are styling only and do not affect
            // layout, so stripping them keeps the structural values readable.
            let mut ir_diagram = diagram_generated.ir_diagram.clone();
            ir_diagram.tailwind_classes = EntityTailwindClasses::default();
            ir_diagram.css = Css::default();
            serde_saphyr::to_fmt_writer(&mut ir_yaml, &ir_diagram)?;
        } else {
            serde_saphyr::to_fmt_writer(&mut ir_yaml, &diagram_generated.ir_diagram)?;
        }
        data_emit(
            output,
            stdout,
            &file_name(file_prefix, "ir_diagram.yaml"),
            &ir_yaml,
        )
        .await?;
    }

    // === SVG elements === //
    // Under `--structure-only` a stripped copy is also used to re-derive the
    // SVG, so the final SVG matches the structure-only SVG elements.
    let svg_elements_structure_only = if structure_only {
        let mut svg_elements = diagram_generated.svg_elements.clone();
        svg_elements.css = Css::default();
        svg_elements.tailwind_classes = EntityTailwindClasses::default();
        svg_elements
            .svg_edge_infos
            .iter_mut()
            .for_each(|svg_edge_info| {
                svg_edge_info.locus_path_d = String::new();
            });
        Some(svg_elements)
    } else {
        None
    };

    if data_is_selected(Data::SvgElements) {
        let svg_elements = svg_elements_structure_only
            .as_ref()
            .unwrap_or(&diagram_generated.svg_elements);
        let mut svg_elements_yaml = String::new();
        serde_saphyr::to_fmt_writer(&mut svg_elements_yaml, svg_elements)?;
        data_emit(
            output,
            stdout,
            &file_name(file_prefix, "svg_elements.yaml"),
            &svg_elements_yaml,
        )
        .await?;
    }

    // === SVG === //
    if data_is_selected(Data::Svg) {
        let svg = match svg_elements_structure_only.as_ref() {
            Some(svg_elements) => SvgElementsToSvgMapper::map_with_input(input_diagram, svg_elements),
            None => diagram_generated.svg.clone(),
        };
        data_emit(output, stdout, &file_name(file_prefix, "diagram.svg"), &svg).await?;
    }

    Ok(())
}

/// Writes the given `contents` to the output directory and/or stdout.
///
/// * `output`: directory to write `file_name` to, if `Some`.
/// * `stdout`: whether to also write `contents` to stdout.
/// * `file_name`: name of the file to write within `output`, e.g.
///   `ir_diagram.yaml`.
/// * `contents`: the data to write.
async fn data_emit(
    output: Option<&Path>,
    stdout: bool,
    file_name: &str,
    contents: &str,
) -> Result<(), CliError> {
    if let Some(output) = output {
        tokio::fs::write(output.join(file_name), contents).await?;
    }
    if stdout {
        print!("{contents}");
        if !contents.ends_with('\n') {
            println!();
        }
    }
    Ok(())
}
