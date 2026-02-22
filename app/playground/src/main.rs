// The dioxus prelude contains a ton of common items used in dioxus apps. It's a
// good idea to import wherever you need dioxus
use dioxus::{
    document::Link,
    prelude::{
        asset, component, dioxus_core, dioxus_signals, manganis, rsx, Asset, Element, Router,
    },
};

use crate::route::Route;

mod components;
mod editor_state;
mod route;
mod views;

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

        // `Router` renders the `Route` enum defined in `route.rs`
        //
        // It will handle synchronization of the URL and render the layouts and components for the active route.
        Router::<Route> {}
    }
}
