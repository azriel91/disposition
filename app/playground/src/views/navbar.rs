use crate::Route;
use dioxus::prelude::{
    component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Link, Outlet,
};

/// The Navbar component that will be rendered on all pages of our app since
/// every page is under the layout.
///
/// This layout component wraps the UI of [Route::Home] and [Route::Blog] in a
/// common navbar. The contents of the Home and Blog routes will be rendered
/// under the outlet inside this component
#[component]
pub fn Navbar() -> Element {
    rsx! {
        div {
            id: "navbar",
            class: "
                flex
                gap-6
                text-lg
                [&>a]:hover:text-blue-300
                [&>a]:transition-colors
                [&>a]:duration-200
            ",
            // components
            Link {
                class: "font-bold",
                to: Route::Home {},
                "📐 disposition"
            }
            Link {
                to: Route::Blog { id: 1 },
                "blog"
            }
        }

        // The `Outlet` component is used to render the next component inside the layout. In this case, it will render either
        // the [`Home`] or [`Blog`] component depending on the current route.
        Outlet::<Route> {}
    }
}
