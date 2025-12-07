use serde::{Deserialize, Serialize};

use crate::process::{ProcessSteps, StepDescs, StepThingInteractions};

/// Represents a process with its steps and associated metadata.
///
/// A process is a grouping of interactions between things sequenced over time.
/// It contains the process name, optional description, steps, step
/// descriptions, and the thing interactions associated with each step.
///
/// # Example
///
/// ```yaml
/// processes:
///   proc_app_dev: # <-- this is a `ProcessDiagram`
///     name: "App Development"
///     desc: |-
///       Development of the web application.
///
///       * [🐙 Repo](https://github.com/azriel91/web_app)
///     steps:
///       proc_app_dev_step_repository_clone: "Clone repository"
///       proc_app_dev_step_project_build: "Build project"
///     step_descs:
///       proc_app_dev_step_repository_clone: |-
///         ```bash
///         git clone https://github.com/azriel91/web_app.git
///         ```
///       proc_app_dev_step_project_build: |-
///         Develop the app:
///
///         * Always link to issue.
///         * Open PR.
///     step_thing_interactions:
///       proc_app_dev_step_repository_clone: [edge_t_localhost__t_github_user_repo__pull]
///       proc_app_dev_step_project_build: [edge_t_localhost__t_localhost__within]
/// ```
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProcessDiagram {
    /// Display name of the process.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Optional description of the process.
    ///
    /// This is intended to take markdown text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,

    /// Steps in the process and their display labels.
    #[serde(default, skip_serializing_if = "ProcessSteps::is_empty")]
    pub steps: ProcessSteps,

    /// Descriptions for each step in the process.
    ///
    /// This is intended to take markdown text.
    #[serde(default, skip_serializing_if = "StepDescs::is_empty")]
    pub step_descs: StepDescs,

    /// Thing interactions that should be actively highlighted when each step is
    /// focused.
    ///
    /// References IDs in `thing_interactions` top level element.
    #[serde(default, skip_serializing_if = "StepThingInteractions::is_empty")]
    pub step_thing_interactions: StepThingInteractions,
}

impl ProcessDiagram {
    /// Returns a new `ProcessDiagram` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a new `ProcessDiagram` with the given name.
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..Default::default()
        }
    }
}
