//! Example diagrams for the playground, served as static assets.
//!
//! Each example is a YAML-serialized
//! [`InputDiagram`](disposition::input_model::InputDiagram) bundled as a static
//! asset via the [`asset!`] macro rather than embedded in the binary. The
//! [`ExampleDiagram`] enum enumerates the available examples and provides
//! accessors for their display labels and asset URLs, plus
//! [`ExampleDiagram::yaml_fetch`] to load the YAML over HTTP at runtime.
//!
//! Serving the diagrams as static files keeps them out of the wasm binary and
//! lets the GitHub Pages deployment (a static file server) serve them directly.

use dioxus::prelude::{asset, manganis, Asset};

pub use self::example_diagram_fetch_error::ExampleDiagramFetchError;

mod example_diagram_fetch_error;

/// An example input diagram that can be loaded into the editor.
///
/// The variants are ordered to introduce features incrementally, from a flat
/// set of nodes through to a capstone that combines everything.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExampleDiagram {
    /// Flat nodes with labels -- `things` and `thing_names`.
    SimpleNodes,
    /// Nesting and layout overrides -- `things` and `thing_layouts`.
    NestedNodes,
    /// Static relationships -- `thing_dependencies` and edge kinds.
    Dependencies,
    /// Runtime communication -- `thing_interactions`.
    Interactions,
    /// Text on edges -- `edge_descs` and `edge_labels`.
    EdgeLabels,
    /// Markdown in descriptions -- headings, emphasis, code, and links.
    Markdown,
    /// Inline images in descriptions via `data:` URLs.
    Images,
    /// Hover detail -- `entity_tooltips`.
    Tooltips,
    /// Colours and strokes -- `entity_types` and theme styles.
    Styling,
    /// Grouping and highlighting things by concern -- `tags`.
    Tags,
    /// Sequenced steps with branching -- `processes`.
    Processes,
    /// Steps that drive interactions -- `step_thing_interactions`.
    ProcessInteractions,
    /// Layout direction and edge curvature -- `render_options`.
    RankDirections,
    /// Applied example: object-oriented class diagram.
    ClassDiagram,
    /// Capstone: a multi-tier cloud architecture combining every feature.
    CloudArchitecture,
    /// Constant-speed interaction animation across very different edge lengths.
    InteractionTiming,
}

impl ExampleDiagram {
    /// All available examples in display order.
    pub const ALL: &'static [ExampleDiagram] = &[
        ExampleDiagram::SimpleNodes,
        ExampleDiagram::NestedNodes,
        ExampleDiagram::Dependencies,
        ExampleDiagram::Interactions,
        ExampleDiagram::EdgeLabels,
        ExampleDiagram::Markdown,
        ExampleDiagram::Images,
        ExampleDiagram::Tooltips,
        ExampleDiagram::Styling,
        ExampleDiagram::Tags,
        ExampleDiagram::Processes,
        ExampleDiagram::ProcessInteractions,
        ExampleDiagram::RankDirections,
        ExampleDiagram::ClassDiagram,
        ExampleDiagram::CloudArchitecture,
        ExampleDiagram::InteractionTiming,
    ];

    /// Human-readable label shown in the example selector dropdown.
    pub fn label(self) -> &'static str {
        match self {
            Self::SimpleNodes => "Simple Nodes",
            Self::NestedNodes => "Nested Nodes",
            Self::Dependencies => "Dependencies",
            Self::Interactions => "Interactions",
            Self::EdgeLabels => "Edge Labels & Descriptions",
            Self::Markdown => "Markdown Descriptions",
            Self::Images => "Inline Images",
            Self::Tooltips => "Tooltips",
            Self::Styling => "Styling",
            Self::Tags => "Tags",
            Self::Processes => "Processes",
            Self::ProcessInteractions => "Processes & Interactions",
            Self::RankDirections => "Rank Directions",
            Self::ClassDiagram => "Class Diagram",
            Self::CloudArchitecture => "Cloud Architecture",
            Self::InteractionTiming => "Interaction Timing",
        }
    }

    /// The static asset for this example's YAML source.
    ///
    /// The returned [`Asset`] resolves to a hashed, base-path-aware URL, e.g.
    /// `/disposition/assets/001_simple_nodes-abc123.yaml`, that is served as a
    /// static file. Use [`Self::url`] to obtain that URL as a string.
    pub fn asset(self) -> Asset {
        match self {
            Self::SimpleNodes => asset!("/assets/example_diagrams/001_simple_nodes.yaml"),
            Self::NestedNodes => asset!("/assets/example_diagrams/002_nested_nodes.yaml"),
            Self::Dependencies => asset!("/assets/example_diagrams/003_dependencies.yaml"),
            Self::Interactions => asset!("/assets/example_diagrams/004_interactions.yaml"),
            Self::EdgeLabels => asset!("/assets/example_diagrams/005_edge_labels.yaml"),
            Self::Markdown => asset!("/assets/example_diagrams/006_markdown.yaml"),
            Self::Images => asset!("/assets/example_diagrams/007_images.yaml"),
            Self::Tooltips => asset!("/assets/example_diagrams/008_tooltips.yaml"),
            Self::Styling => asset!("/assets/example_diagrams/009_styling.yaml"),
            Self::Tags => asset!("/assets/example_diagrams/010_tags.yaml"),
            Self::Processes => asset!("/assets/example_diagrams/011_processes.yaml"),
            Self::ProcessInteractions => {
                asset!("/assets/example_diagrams/012_process_interactions.yaml")
            }
            Self::RankDirections => {
                asset!("/assets/example_diagrams/013_rank_directions.yaml")
            }
            Self::ClassDiagram => asset!("/assets/example_diagrams/014_class_diagram.yaml"),
            Self::CloudArchitecture => {
                asset!("/assets/example_diagrams/015_cloud_architecture.yaml")
            }
            Self::InteractionTiming => {
                asset!("/assets/example_diagrams/016_interaction_timing.yaml")
            }
        }
    }

    /// The served URL for this example's YAML source.
    ///
    /// e.g. `/disposition/assets/001_simple_nodes-abc123.yaml`.
    pub fn url(self) -> String {
        self.asset().to_string()
    }

    /// Fetches the raw YAML source for this example over HTTP.
    ///
    /// The YAML files are served as static assets, so the source is requested
    /// from [`Self::url`] at runtime rather than embedded in the binary.
    pub async fn yaml_fetch(self) -> Result<String, ExampleDiagramFetchError> {
        let url = self.url();

        #[cfg(target_family = "wasm")]
        {
            self.yaml_fetch_wasm(url).await
        }

        #[cfg(not(target_family = "wasm"))]
        {
            Err(ExampleDiagramFetchError::Unsupported {
                example_diagram: self,
                url,
            })
        }
    }

    /// Fetches the YAML source using the browser's `fetch` API.
    #[cfg(target_family = "wasm")]
    async fn yaml_fetch_wasm(self, url: String) -> Result<String, ExampleDiagramFetchError> {
        use gloo_net::http::Request;

        let response = Request::get(&url).send().await.map_err(|error| {
            ExampleDiagramFetchError::RequestSend {
                example_diagram: self,
                url: url.clone(),
                error: error.to_string(),
            }
        })?;

        if !response.ok() {
            return Err(ExampleDiagramFetchError::ResponseStatus {
                example_diagram: self,
                url,
                status: response.status(),
            });
        }

        response
            .text()
            .await
            .map_err(|error| ExampleDiagramFetchError::ResponseText {
                example_diagram: self,
                url,
                error: error.to_string(),
            })
    }

    /// Look up an example by its zero-based index in [`Self::ALL`].
    ///
    /// Returns `None` when the index is out of range.
    pub fn from_index(index: usize) -> Option<ExampleDiagram> {
        Self::ALL.get(index).copied()
    }
}
