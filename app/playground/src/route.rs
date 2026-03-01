use dioxus::{
    prelude::{dioxus_core, dioxus_router, dioxus_signals, rsx, VNode},
    router::Routable,
};

use crate::{
    editor_state::EditorState,
    views::{Blog, Home, Navbar},
};

/// The Route enum is used to define the structure of internal routes in our
/// app. All route enums need to derive the [`Routable`] trait, which provides
/// the necessary methods for the router to work.
///
/// Each variant represents a different URL pattern that can be matched by the
/// router. If that pattern is matched, the components for that route will be
/// rendered.
#[derive(Debug, Clone, Routable, PartialEq)]
#[allow(clippy::large_enum_variant)] // TODO: `EditorState` is large (1328 bytes), we need to box things inside it.
pub enum Route {
    // The layout attribute defines a wrapper for all routes under the layout. Layouts are great
    // for wrapping many routes with a common UI like a navbar.
    #[layout(Navbar)]
    // The route attribute defines the URL pattern that a specific route matches. If that pattern
    // matches the URL, the component for that route will be rendered. The component name that
    // is rendered defaults to the variant name.
    #[route("/#:editor_state")]
    Home {
        /// The editor state, serialized as YAML in the URL hash fragment.
        ///
        /// This captures the full editor state including the active page/tab
        /// and the [`InputDiagram`](disposition::input_model::InputDiagram)
        /// being edited.
        editor_state: EditorState,
    },
    // The route attribute can include dynamic parameters that implement [`std::str::FromStr`] and
    // [`std::fmt::Display`] with the `:` syntax. In this case, id will match any integer like
    // `/blog/123` or `/blog/-456`.
    #[route("/blog/:id")]
    // Fields of the route variant will be passed to the component as props. In this case, the blog
    // component must accept an `id` prop of type `i32`.
    Blog { id: i32 },
}
