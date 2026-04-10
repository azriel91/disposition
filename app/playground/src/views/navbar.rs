use crate::{editor_state::EditorState, hooks::use_dark_mode_provider, Route};
use dioxus::prelude::{
    component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Link, Outlet,
};

use super::theme_toggle::ThemeToggle;

/// The `Navbar` component that will be rendered on all pages of our app since
/// every page is under the layout.
///
/// This layout component wraps the UI of [Route::Home] in a common navbar. The
/// contents of the Home route will be rendered under the outlet inside this
/// component.
#[component]
pub fn Navbar() -> Element {
    // Provide the shared dark-mode signal for all descendant components.
    let _dark_mode = use_dark_mode_provider();

    rsx! {
        div {
            id: "navbar",
            class: "
                flex
                items-center
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
                to: Route::Licenses,
                "📝 licenses"
            }
            Link {
                to: "https://github.com/azriel91/disposition",
                new_tab: true,
                "🐙 github"
            }

            // Spacer to push the theme toggle to the right.
            div { class: "flex-1" }

            ThemeToggle {}
        }

        // The `Outlet` component is used to render the next component inside
        // the layout.
        //
        // In this case, it will render the [`Home`] component since that's the
        // only route defined.
        Outlet::<Route> {}
    }
}
