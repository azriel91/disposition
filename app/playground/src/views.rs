//! Contains the UI for all Layouts and Routes for our app.
//!
//! The views module contains the components for all Layouts and Routes for our
//! app. Each layout and route in our [`Route`] enum will render one of these
//! components.
//!
//! The [`Home`] and [`Blog`] components will be rendered when the current route
//! is [`Route::Home`] or [`Route::Blog`] respectively.
//!
//! The [`Navbar`] component will be rendered on all pages of our app since
//! every page is under the layout. The layout defines a common wrapper around
//! all child routes.

pub use self::{blog::Blog, home::Home, navbar::Navbar};

mod blog;
mod home;
mod navbar;
