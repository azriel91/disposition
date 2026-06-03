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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExampleDiagram {
    /// Simple flat nodes with no hierarchy or edges.
    SimpleNodes,
    /// Nested nodes demonstrating `thing_hierarchy`.
    NestedNodes,
    /// Nodes with dependency edges of different kinds.
    Dependencies,
    /// Interactions between things wired into sequenced processes.
    InteractionsAndProcesses,
    /// Tags for grouping and highlighting things by concern.
    Tags,
    /// Entity types and theme styles for visual customization.
    Themed,
    /// A process definition with no things at all.
    ProcessOnly,
    /// Object-oriented class diagram with inheritance and composition.
    ClassDiagram,
    /// Software package dependency graph.
    PackageDependencies,
    /// CI/CD deployment pipeline with interactions and process steps.
    DeploymentProcess,
    /// Multi-tier cloud network architecture.
    CloudArchitecture,
}

impl ExampleDiagram {
    /// All available examples in display order.
    pub const ALL: &'static [ExampleDiagram] = &[
        ExampleDiagram::SimpleNodes,
        ExampleDiagram::NestedNodes,
        ExampleDiagram::Dependencies,
        ExampleDiagram::InteractionsAndProcesses,
        ExampleDiagram::Tags,
        ExampleDiagram::Themed,
        ExampleDiagram::ProcessOnly,
        ExampleDiagram::ClassDiagram,
        ExampleDiagram::PackageDependencies,
        ExampleDiagram::DeploymentProcess,
        ExampleDiagram::CloudArchitecture,
    ];

    /// Human-readable label shown in the example selector dropdown.
    pub fn label(self) -> &'static str {
        match self {
            Self::SimpleNodes => "Simple Nodes",
            Self::NestedNodes => "Nested Nodes",
            Self::Dependencies => "Dependencies",
            Self::InteractionsAndProcesses => "Interactions & Processes",
            Self::Tags => "Tags",
            Self::Themed => "Themed",
            Self::ProcessOnly => "Process Only (no things)",
            Self::ClassDiagram => "Class Diagram",
            Self::PackageDependencies => "Package Dependencies",
            Self::DeploymentProcess => "Deployment Process",
            Self::CloudArchitecture => "Cloud Architecture",
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
            Self::InteractionsAndProcesses => {
                asset!("/assets/example_diagrams/004_interactions_and_processes.yaml")
            }
            Self::Tags => asset!("/assets/example_diagrams/005_tags.yaml"),
            Self::Themed => asset!("/assets/example_diagrams/006_themed.yaml"),
            Self::ProcessOnly => asset!("/assets/example_diagrams/007_process_only.yaml"),
            Self::ClassDiagram => asset!("/assets/example_diagrams/008_class_diagram.yaml"),
            Self::PackageDependencies => {
                asset!("/assets/example_diagrams/009_package_dependencies.yaml")
            }
            Self::DeploymentProcess => {
                asset!("/assets/example_diagrams/010_deployment_process.yaml")
            }
            Self::CloudArchitecture => {
                asset!("/assets/example_diagrams/011_cloud_architecture.yaml")
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
