// The dioxus prelude contains a ton of common items used in dioxus apps. It's a
// good idea to import wherever you need dioxus
use dioxus::{
    document::{Link, Script},
    prelude::{
        asset, component, dioxus_core, dioxus_signals, manganis, rsx, Asset, Element, Router,
    },
};

use crate::route::Route;

mod components;
mod editor_state;
mod example_diagrams;
mod hooks;
mod route;
mod undo_history;
mod views;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styling/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

/// Inline script that runs before the first paint to apply the `.dark` class
/// on `<html>` if the user previously chose dark mode (or if the OS prefers
/// dark). This prevents a flash of the wrong colour scheme.
const DARK_MODE_INIT_SCRIPT: &str = r#"
(function() {
    var stored = localStorage.getItem("disposition-dark-mode");
    var dark;
    if (stored === "true") {
        dark = true;
    } else if (stored === "false") {
        dark = false;
    } else {
        dark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    }
    if (dark) {
        document.documentElement.classList.add("dark");
    } else {
        document.documentElement.classList.remove("dark");
    }
})();
"#;

fn main() {
    dioxus::launch(App);
}

/// `App` is the main component of our app.
#[component]
fn App() -> Element {
    rsx! {
        // Apply the dark-mode class before stylesheets paint so there is no
        // flash of the wrong colour scheme on page load.
        Script { {DARK_MODE_INIT_SCRIPT} }

        Link { rel: "icon", href: FAVICON }
        Link { rel: "stylesheet", href: MAIN_CSS }
        Link { rel: "stylesheet", href: TAILWIND_CSS }

        // `Router` renders the `Route` enum defined in `route.rs`
        //
        // It will handle synchronization of the URL and render the layouts and components for the active route.
        Router::<Route> {}
    }
}
