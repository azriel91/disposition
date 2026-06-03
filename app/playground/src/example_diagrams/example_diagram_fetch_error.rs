//! Errors that can occur when fetching an example diagram's YAML over HTTP.

use std::{error::Error, fmt};

use crate::example_diagrams::ExampleDiagram;

/// An error that occurred while fetching the YAML for an [`ExampleDiagram`].
///
/// The example diagram YAML files are served as static assets, so loading one
/// requires an HTTP request. Each variant captures the [`ExampleDiagram`] and
/// the `url` that was requested so the failure can be reported meaningfully.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExampleDiagramFetchError {
    /// The HTTP request to fetch the example diagram YAML could not be sent.
    #[cfg(target_family = "wasm")]
    RequestSend {
        /// The example diagram whose YAML was being fetched.
        example_diagram: ExampleDiagram,
        /// The static asset URL that was requested, e.g.
        /// `/disposition/assets/001_simple_nodes-abc123.yaml`.
        url: String,
        /// The underlying error message from the HTTP client.
        error: String,
    },
    /// The server returned a non-success HTTP status code.
    #[cfg(target_family = "wasm")]
    ResponseStatus {
        /// The example diagram whose YAML was being fetched.
        example_diagram: ExampleDiagram,
        /// The static asset URL that was requested.
        url: String,
        /// The HTTP status code returned by the server, e.g. `404`.
        status: u16,
    },
    /// The response body could not be read as text.
    #[cfg(target_family = "wasm")]
    ResponseText {
        /// The example diagram whose YAML was being fetched.
        example_diagram: ExampleDiagram,
        /// The static asset URL that was requested.
        url: String,
        /// The underlying error message from the HTTP client.
        error: String,
    },
    /// Fetching is not supported on the current build target.
    ///
    /// The playground only fetches example diagrams on the `wasm` target;
    /// other targets (used for tests / docs) cannot perform the request.
    #[cfg(not(target_family = "wasm"))]
    Unsupported {
        /// The example diagram whose YAML was being fetched.
        example_diagram: ExampleDiagram,
        /// The static asset URL that would have been requested.
        url: String,
    },
}

impl fmt::Display for ExampleDiagramFetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(target_family = "wasm")]
            Self::RequestSend {
                example_diagram,
                url,
                error,
            } => write!(
                f,
                "Failed to request example diagram `{label}` from `{url}`: {error}",
                label = example_diagram.label(),
            ),
            #[cfg(target_family = "wasm")]
            Self::ResponseStatus {
                example_diagram,
                url,
                status,
            } => write!(
                f,
                "Request for example diagram `{label}` from `{url}` \
                returned status `{status}`.",
                label = example_diagram.label(),
            ),
            #[cfg(target_family = "wasm")]
            Self::ResponseText {
                example_diagram,
                url,
                error,
            } => write!(
                f,
                "Failed to read response body for example diagram `{label}` \
                from `{url}`: {error}",
                label = example_diagram.label(),
            ),
            #[cfg(not(target_family = "wasm"))]
            Self::Unsupported {
                example_diagram,
                url,
            } => write!(
                f,
                "Fetching example diagram `{label}` from `{url}` is not \
                supported on this build target.",
                label = example_diagram.label(),
            ),
        }
    }
}

impl Error for ExampleDiagramFetchError {}
