pub use self::tab_details::TabDetails;

mod tab_details;

use dioxus::prelude::{
    component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props,
};

/// Static CSS `id` values for tab inputs, listed so the Tailwind CSS scanner
/// detects them alongside their corresponding `group-[:has(...)]` panel
/// classes.
const TAB_IDS: &[&str] = &[
    "tab-0", "tab-1", "tab-2", "tab-3", "tab-4", "tab-5", "tab-6", "tab-7",
];

/// Static CSS class strings for tab panel visibility.
///
/// Each entry pairs with the same-index [`TAB_IDS`] element via
/// `group-[:has(#tab-N:checked)]:flex`.
const TAB_PANEL_CLASSES: &[&str] = &[
    "hidden group-[:has(#tab-0:checked)]:flex flex-col",
    "hidden group-[:has(#tab-1:checked)]:flex flex-col",
    "hidden group-[:has(#tab-2:checked)]:flex flex-col",
    "hidden group-[:has(#tab-3:checked)]:flex flex-col",
    "hidden group-[:has(#tab-4:checked)]:flex flex-col",
    "hidden group-[:has(#tab-5:checked)]:flex flex-col",
    "hidden group-[:has(#tab-6:checked)]:flex flex-col",
    "hidden group-[:has(#tab-7:checked)]:flex flex-col",
];

/// A CSS-only tab group component.
///
/// **Supports up to 8 tabs. Additional entries beyond 8 are silently ignored.**
///
/// Uses Tailwind CSS `group` with `:has(#tab-N:checked)` variants to toggle
/// panel visibility without JavaScript. Each tab is a `<label>` wrapping a
/// hidden `<input type="radio">` with a static `id` (`tab-0`~`tab-7`), and
/// each panel `<div>` uses the corresponding
/// `group-[:has(#tab-N:checked)]:flex` class to become visible when its radio
/// input is selected.
///
/// Because the visibility selector goes through a `group` ancestor (rather than
/// the sibling-only `peer`), the labels and panels do **not** need to be
/// adjacent siblings -- they can live in separate wrapper `<div>`s.
///
/// # Parameters
///
/// * `group_name`: The `name` attribute shared by all radio inputs.
/// * `tabs`: A `Vec<TabDetails>` describing each tab.
/// * `default_checked`: Optional index of the tab that starts checked.
#[component]
pub fn TabGroup(
    group_name: String,
    tabs: Vec<TabDetails>,
    #[props(default)] default_checked: Option<usize>,
) -> Element {
    let tab_count = tabs.len().min(TAB_IDS.len());

    rsx! {
        div {
            class: "group flex flex-col",

            // ── Tab label row ────────────────────────────────────────
            div {
                class: "
                    flex
                    flex-row
                    gap-1
                    border-b
                    border-gray-700
                    mb-2
                ",

                for i in 0..tab_count {
                    {
                        let tab_id = TAB_IDS[i];
                        let label_text = tabs[i].label.clone();
                        let is_default = default_checked == Some(i);
                        let group = group_name.clone();

                        rsx! {
                            label {
                                class: "
                                    cursor-pointer
                                    select-none
                                    px-4
                                    py-2
                                    text-sm
                                    font-semibold
                                    text-gray-400
                                    border-b-2
                                    border-transparent
                                    transition-colors
                                    duration-150
                                    hover:text-gray-200
                                    has-[:checked]:text-blue-400
                                    has-[:checked]:border-blue-400
                                ",
                                input {
                                    r#type: "radio",
                                    id: "{tab_id}",
                                    name: "{group}",
                                    class: "sr-only",
                                    checked: if is_default { true },
                                }
                                "{label_text}"
                            }
                        }
                    }
                }
            }

            // ── Tab panels ───────────────────────────────────────────
            for i in 0..tab_count {
                {
                    let panel_class = TAB_PANEL_CLASSES[i];
                    let content = tabs[i].content.clone();

                    rsx! {
                        div {
                            class: "{panel_class}",
                            {content}
                        }
                    }
                }
            }
        }
    }
}
