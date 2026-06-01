use std::path::PathBuf;

use clap::Parser;
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
/// Writes the following files to the output directory:
///
/// * `ir_diagram.yaml`: the intermediate representation diagram
/// * `taffy_tree.txt`: the taffy layout tree
/// * `svg_elements.yaml`: the SVG elements
/// * `diagram.svg`: the final SVG
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Path to the input diagram YAML file.
    input: PathBuf,
    /// Directory to write output files to.
    output: PathBuf,
    /// Only output values relevant to the structure of the diagram, without any
    /// styles or colors.
    #[arg(long)]
    structure_only: bool,
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
    } = Args::parse();

    let contents = tokio::fs::read_to_string(&input).await?;
    let input_diagram: InputDiagram<'static> = serde_saphyr::from_str(&contents)?;

    tokio::fs::create_dir_all(&output).await?;

    let input_diagram_merged = InputDiagramMerger::merge(InputDiagram::base(), &input_diagram);
    let ir_diagram_and_issues = InputToIrDiagramMapper::map(&input_diagram_merged);

    if !ir_diagram_and_issues.issues.is_empty() {
        eprintln!("Issues mapping input to IR diagram:");
        for issue in &ir_diagram_and_issues.issues {
            eprintln!("  {issue}");
        }
    }

    let ir_diagram = if structure_only {
        let mut ir_diagram = ir_diagram_and_issues.diagram;
        ir_diagram.tailwind_classes = EntityTailwindClasses::default();
        ir_diagram.css = Css::default();
        ir_diagram
    } else {
        ir_diagram_and_issues.diagram
    };

    let mut ir_yaml = String::new();
    serde_saphyr::to_fmt_writer(&mut ir_yaml, &ir_diagram)?;
    tokio::fs::write(output.join("ir_diagram.yaml"), &ir_yaml).await?;

    let taffy_node_mappings = IrToTaffyBuilder::builder()
        .with_ir_diagram(&ir_diagram)
        .with_dimension_and_lods(vec![DimensionAndLod::default_no_limit()])
        .build()
        .build()?
        .next()
        .ok_or(CliError::NoTaffyMappings)?;

    let mut taffy_tree = String::new();
    TaffyTreeFmt::fmt(&mut taffy_tree, &taffy_node_mappings);
    tokio::fs::write(output.join("taffy_tree.txt"), &taffy_tree).await?;

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

    let mut svg_elements_yaml = String::new();
    serde_saphyr::to_fmt_writer(&mut svg_elements_yaml, &svg_elements)?;
    tokio::fs::write(output.join("svg_elements.yaml"), &svg_elements_yaml).await?;

    let svg = SvgElementsToSvgMapper::map_with_input(&input_diagram, &svg_elements);
    tokio::fs::write(output.join("diagram.svg"), svg).await?;

    Ok(())
}
