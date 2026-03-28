//! Dark-mode toggle hook with `localStorage` persistence.
//!
//! Provides [`use_dark_mode`], which returns a reactive [`Signal<bool>`] that
//! is `true` when dark mode is active. Toggling the signal automatically
//! adds/removes the `.dark` class on `<html>` and persists the choice to
//! `localStorage` under the key `"disposition-dark-mode"`.
//!
//! On first load the hook checks (in order):
//!
//! 1. `localStorage["disposition-dark-mode"]` -- explicit user choice.
//! 2. `prefers-color-scheme: dark` media query -- OS preference.
//! 3. Falls back to **dark** if neither is available.

use dioxus::{
    hooks::{use_effect, use_signal},
    prelude::document,
    signals::{ReadableExt, Signal, WritableExt},
};

/// `localStorage` key used to persist the user's theme choice.
const STORAGE_KEY: &str = "disposition-dark-mode";

/// Initialise (or retrieve) the dark-mode state for the current component
/// tree.
///
/// The returned [`Signal<bool>`] is `true` when dark mode is active.
///
/// # Behaviour
///
/// * **First render**: reads `localStorage` / `prefers-color-scheme` and
///   synchronises the `<html>` element's class list.
/// * **Subsequent writes**: any write to the signal (e.g. via
///   [`dark_mode_toggle`]) triggers a `use_effect` that updates the DOM and
///   `localStorage`.
///
/// # Example
///
/// ```rust,ignore
/// let dark_mode = use_dark_mode();
/// // Toggle:
/// dark_mode_toggle(dark_mode);
/// ```
pub fn use_dark_mode() -> Signal<bool> {
    // Start with `true` (dark) as a safe default; the effect below will
    // correct it on the very first frame once JS has executed.
    let mut is_dark = use_signal(|| true);

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

/// Toggle the dark-mode signal (convenience helper).
pub fn dark_mode_toggle(mut is_dark: Signal<bool>) {
    let current = *is_dark.peek();
    is_dark.set(!current);
}
