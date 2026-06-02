use std::path::{Path, PathBuf};

use clap::{Parser, ValueEnum};
use disposition::{
    input_model::InputDiagram,
    ir_model::entity::EntityTailwindClasses,
    model_common::theme::Css,
    taffy_model::{DimensionAndLod, IrToTaffyError, TaffyTreeFmt},
};
use disposition_input_ir_rt::{
    EdgeAnimationActive, InputDiagramMerger, InputToIrDiagramMapper, IrToTaffyBuilder,
    SvgElementsToSvgMapper, TaffyToSvgElementsMapper,
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
    #[error("taffy: {0}")]
    Taffy(#[from] IrToTaffyError),
    #[error("no taffy node mappings generated")]
    NoTaffyMappings,
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
    let stage_max = data_selected.unwrap_or(Data::Svg);
    let data_is_selected =
        |data: Data| data_selected.is_none_or(|data_selected| data_selected == data);

    let contents = tokio::fs::read_to_string(&input).await?;
    let input_diagram: InputDiagram<'static> = serde_saphyr::from_str(&contents)?;

    if let Some(output) = output.as_deref() {
        tokio::fs::create_dir_all(output).await?;
    }
    let output = output.as_deref();

    let input_diagram_merged = InputDiagramMerger::merge(InputDiagram::base(), &input_diagram);
    let ir_diagram_and_issues = InputToIrDiagramMapper::map(&input_diagram_merged);

    if !ir_diagram_and_issues.issues.is_empty() {
        eprintln!("Issues mapping input to IR diagram:");
        for issue in &ir_diagram_and_issues.issues {
            eprintln!("  {issue}");
        }
    }

    // === IR diagram === //
    let ir_diagram = if structure_only {
        let mut ir_diagram = ir_diagram_and_issues.diagram;
        ir_diagram.tailwind_classes = EntityTailwindClasses::default();
        ir_diagram.css = Css::default();
        ir_diagram
    } else {
        ir_diagram_and_issues.diagram
    };

    if data_is_selected(Data::IrDiagram) {
        let mut ir_yaml = String::new();
        serde_saphyr::to_fmt_writer(&mut ir_yaml, &ir_diagram)?;
        data_emit(output, stdout, "ir_diagram.yaml", &ir_yaml).await?;
    }
    if stage_max == Data::IrDiagram {
        return Ok(());
    }

    // === Taffy tree === //
    let taffy_node_mappings = IrToTaffyBuilder::builder()
        .with_ir_diagram(&ir_diagram)
        .with_dimension_and_lods(vec![DimensionAndLod::default_no_limit()])
        .build()
        .build()?
        .next()
        .ok_or(CliError::NoTaffyMappings)?;

    if data_is_selected(Data::TaffyTree) {
        let mut taffy_tree = String::new();
        TaffyTreeFmt::fmt(&mut taffy_tree, &taffy_node_mappings);
        data_emit(output, stdout, "taffy_tree.txt", &taffy_tree).await?;
    }
    if stage_max == Data::TaffyTree {
        return Ok(());
    }

    // === SVG elements === //
    let mut svg_elements = TaffyToSvgElementsMapper::map(
        &ir_diagram,
        &taffy_node_mappings,
        EdgeAnimationActive::OnProcessStepFocus,
    );

    if structure_only {
        svg_elements.css = Css::default();
        svg_elements.tailwind_classes = EntityTailwindClasses::default();
        svg_elements
            .svg_edge_infos
            .iter_mut()
            .for_each(|svg_edge_info| {
                svg_edge_info.locus_path_d = String::new();
            });
    }

    if data_is_selected(Data::SvgElements) {
        let mut svg_elements_yaml = String::new();
        serde_saphyr::to_fmt_writer(&mut svg_elements_yaml, &svg_elements)?;
        data_emit(output, stdout, "svg_elements.yaml", &svg_elements_yaml).await?;
    }
    if stage_max == Data::SvgElements {
        return Ok(());
    }

    // === SVG === //
    let svg = SvgElementsToSvgMapper::map_with_input(&input_diagram, &svg_elements);
    data_emit(output, stdout, "diagram.svg", &svg).await?;

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
