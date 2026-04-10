//! Contains the UI for all Layouts and Routes for our app.
//!
//! The views module contains the components for all Layouts and Routes for our
//! app. Each layout and route in our [`Route`] enum will render one of these
//! components.
//!
//! The [`Home`] component will be rendered when the current route is
//! [`Route::Home`].
//!
//! The [`Navbar`] component will be rendered on all pages of our app since
//! every page is under the layout. The layout defines a common wrapper around
//! all child routes.
//!
//! [`Route`]: crate::Route
//! [`Route::Home`]: crate::Route::Home

pub use self::{home::Home, licenses::Licenses, navbar::Navbar};

mod home;
mod licenses;
mod navbar;
pub(crate) mod theme_toggle;
