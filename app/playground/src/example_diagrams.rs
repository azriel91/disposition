//! Embedded example diagrams for the playground.
//!
//! Each example is a YAML-serialized
//! [`InputDiagram`](disposition::input_model::InputDiagram) included at compile
//! time via `include_str!`. The [`ExampleDiagram`] enum enumerates the
//! available examples and provides accessors for their display labels and YAML
//! content.

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

    /// The raw YAML source for this example, embedded at compile time.
    pub fn yaml(self) -> &'static str {
        match self {
            Self::SimpleNodes => {
                include_str!("example_diagrams/001_simple_nodes.yaml")
            }
            Self::NestedNodes => {
                include_str!("example_diagrams/002_nested_nodes.yaml")
            }
            Self::Dependencies => {
                include_str!("example_diagrams/003_dependencies.yaml")
            }
            Self::InteractionsAndProcesses => {
                include_str!("example_diagrams/004_interactions_and_processes.yaml")
            }
            Self::Tags => {
                include_str!("example_diagrams/005_tags.yaml")
            }
            Self::Themed => {
                include_str!("example_diagrams/006_themed.yaml")
            }
            Self::ProcessOnly => {
                include_str!("example_diagrams/007_process_only.yaml")
            }
            Self::ClassDiagram => {
                include_str!("example_diagrams/008_class_diagram.yaml")
            }
            Self::PackageDependencies => {
                include_str!("example_diagrams/009_package_dependencies.yaml")
            }
            Self::DeploymentProcess => {
                include_str!("example_diagrams/010_deployment_process.yaml")
            }
            Self::CloudArchitecture => {
                include_str!("example_diagrams/011_cloud_architecture.yaml")
            }
        }
    }

    /// Look up an example by its zero-based index in [`Self::ALL`].
    ///
    /// Returns `None` when the index is out of range.
    pub fn from_index(index: usize) -> Option<ExampleDiagram> {
        Self::ALL.get(index).copied()
    }
}
