// The dioxus prelude contains a ton of common items used in dioxus apps. It's a
// good idea to import wherever you need dioxus
use dioxus::{
    document::Link,
    prelude::{
        asset, component, dioxus_core, dioxus_router, dioxus_signals, manganis, rsx, Asset,
        Element, Routable, Router, VNode,
    },
};

use views::{Blog, Home, Navbar};

mod components;
mod views;

/// The Route enum is used to define the structure of internal routes in our
/// app. All route enums need to derive the [`Routable`] trait, which provides
/// the necessary methods for the router to work.
///
/// Each variant represents a different URL pattern that can be matched by the
/// router. If that pattern is matched, the components for that route will be
/// rendered.
#[derive(Debug, Clone, Routable, PartialEq)]
enum Route {
    // The layout attribute defines a wrapper for all routes under the layout. Layouts are great
    // for wrapping many routes with a common UI like a navbar.
    #[layout(Navbar)]
    // The route attribute defines the URL pattern that a specific route matches. If that pattern
    // matches the URL, the component for that route will be rendered. The component name that
    // is rendered defaults to the variant name.
    #[route("/")]
    Home {},
    // The route attribute can include dynamic parameters that implement [`std::str::FromStr`] and
    // [`std::fmt::Display`] with the `:` syntax. In this case, id will match any integer like
    // `/blog/123` or `/blog/-456`.
    #[route("/blog/:id")]
    // Fields of the route variant will be passed to the component as props. In this case, the blog
    // component must accept an `id` prop of type `i32`.
    Blog { id: i32 },
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

/// `App` is the main component of our app.
#[component]
fn App() -> Element {
    rsx! {
        Link { rel: "icon", href: FAVICON }
        Link { rel: "stylesheet", href: MAIN_CSS }
        Link { rel: "stylesheet", href: TAILWIND_CSS }

        // `Router` renders the `Route` enum we defined above.
        //
        // It will handle synchronization of the URL and render the layouts and components for the active route.
        Router::<Route> {}
    }
}
