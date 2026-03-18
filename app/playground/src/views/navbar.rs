use crate::{editor_state::EditorState, Route};
use dioxus::prelude::{
    component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Link, Outlet,
};

/// The `Navbar` component that will be rendered on all pages of our app since
/// every page is under the layout.
///
/// This layout component wraps the UI of [Route::Home] in a common navbar. The
/// contents of the Home route will be rendered under the outlet inside this
/// component.
#[component]
pub fn Navbar() -> Element {
    rsx! {
        div {
            id: "navbar",
            class: "
                flex
                gap-6
                [&>a]:hover:text-blue-300
                [&>a]:transition-colors
                [&>a]:duration-100
            ",
            Link {
                class: "\
                    text-lg \
                    font-bold\
                ",
                to: Route::Home { editor_state: EditorState::default() },
                "📐 disposition"
            }
            Link {
                to: "https://github.com/azriel91/disposition",
                new_tab: true,
                "github"
            }
        }

        // The `Outlet` component is used to render the next component inside
        // the layout.
        //
        // In this case, it will render the [`Home`] component since that's the
        // only route defined.
        Outlet::<Route> {}
    }
}
