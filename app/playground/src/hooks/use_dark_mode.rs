//! Dark-mode toggle hook with `localStorage` persistence.
//!
//! Provides [`use_dark_mode_provider`] and [`use_dark_mode`] for sharing a
//! single reactive dark-mode [`Signal<bool>`] across the entire component
//! tree via the Dioxus Context API.
//!
//! # Setup
//!
//! Call [`use_dark_mode_provider`] **once** near the root of the component
//! tree (e.g. in a layout component that wraps all routes). Every descendant
//! component that needs the current dark-mode state should call
//! [`use_dark_mode`], which retrieves the shared signal via
//! [`use_context`].
//!
//! # Behaviour
//!
//! On first load the provider checks (in order):
//!
//! 1. `localStorage["disposition-dark-mode"]` -- explicit user choice.
//! 2. `prefers-color-scheme: dark` media query -- OS preference.
//! 3. Falls back to **dark** if neither is available.
//!
//! Subsequent writes to the signal (e.g. via [`dark_mode_toggle`])
//! automatically add/remove the `.dark` class on `<html>` and persist the
//! choice to `localStorage`.

use dioxus::{
    hooks::{use_context, use_context_provider, use_effect, use_signal},
    prelude::document,
    signals::{ReadableExt, Signal, WritableExt},
};

/// `localStorage` key used to persist the user's theme choice.
const STORAGE_KEY: &str = "disposition-dark-mode";

/// Create and provide the shared dark-mode signal for the component tree.
///
/// This must be called **once** in a component that is an ancestor of every
/// component that calls [`use_dark_mode`]. The layout component that wraps
/// all routes (e.g. `Navbar`) is a good place.
///
/// Returns the same [`Signal<bool>`] that descendants will receive from
/// [`use_dark_mode`].
///
/// # Example
///
/// ```rust,ignore
/// #[component]
/// pub fn Navbar() -> Element {
///     let _dark_mode = use_dark_mode_provider();
///     rsx! { Outlet::<Route> {} }
/// }
/// ```
pub fn use_dark_mode_provider() -> Signal<bool> {
    // Start with `true` (dark) as a safe default; the init effect below
    // will correct it on the very first frame once JS has executed.
    let mut is_dark: Signal<bool> = use_signal(|| true);
    use_context_provider(|| is_dark);

    // --- One-shot initialisation --- //
    //
    // We cannot read localStorage synchronously from Rust, so we eval a
    // small JS snippet that reads the stored preference (or falls back to
    // the OS media query) and sends the result back.
    //
    // The snippet also applies the `.dark` class immediately so there is no
    // flash of the wrong theme.
    let mut initialised = use_signal(|| false);

    use_effect(move || {
        if *initialised.peek() {
            return;
        }
        initialised.set(true);

        let js = format!(
            r#"(() => {{
                let stored = localStorage.getItem("{key}");
                let dark;
                if (stored === "true") {{
                    dark = true;
                }} else if (stored === "false") {{
                    dark = false;
                }} else {{
                    dark = window.matchMedia("(prefers-color-scheme: dark)").matches;
                }}
                if (dark) {{
                    document.documentElement.classList.add("dark");
                }} else {{
                    document.documentElement.classList.remove("dark");
                }}
                dioxus.send(dark);
            }})()"#,
            key = STORAGE_KEY,
        );

        dioxus::core::spawn(async move {
            if let Ok(val) = document::eval(&js).recv::<bool>().await {
                is_dark.set(val);
            }
        });
    });

    // --- Keep DOM + localStorage in sync on every change --- //
    use_effect(move || {
        let dark = *is_dark.read();

        // Skip the very first effect run that happens before the init
        // effect has had a chance to send back the real value.
        if !*initialised.peek() {
            return;
        }

        let js = format!(
            r#"(() => {{
                localStorage.setItem("{key}", {dark});
                if ({dark}) {{
                    document.documentElement.classList.add("dark");
                }} else {{
                    document.documentElement.classList.remove("dark");
                }}
            }})()"#,
            key = STORAGE_KEY,
            dark = dark,
        );

        document::eval(&js);
    });

    is_dark
}

/// Retrieve the shared dark-mode signal from context.
///
/// The returned [`Signal<bool>`] is `true` when dark mode is active.
///
/// # Panics
///
/// Panics if [`use_dark_mode_provider`] has not been called in an ancestor
/// component.
///
/// # Example
///
/// ```rust,ignore
/// let is_dark = use_dark_mode();
/// if is_dark() { /* dark theme */ }
/// ```
pub fn use_dark_mode() -> Signal<bool> {
    use_context::<Signal<bool>>()
}

/// Toggle the dark-mode signal (convenience helper).
pub fn dark_mode_toggle(mut is_dark: Signal<bool>) {
    let current = *is_dark.peek();
    is_dark.set(!current);
}
