//! A single card within the theme styles editor.
//!
//! Composes the header, style aliases, and theme attributes sub-components
//! into one card per `IdOrDefaults -> CssClassPartials` entry.

use dioxus::{
    prelude::{component, dioxus_core, dioxus_elements, dioxus_signals, rsx, Element, Props},
    signals::Signal,
};
use disposition::input_model::InputDiagram;

use crate::components::editor::{
    common::CARD_CLASS,
    theme_styles_editor::{
        css_class_partials_card_aliases::CssClassPartialsCardAliases,
        css_class_partials_card_attrs::CssClassPartialsCardAttrs,
        css_class_partials_card_header::CssClassPartialsCardHeader,
        theme_attr_entry::ThemeAttrEntry, ThemeStylesTarget,
    },
};

// === CssClassPartialsCard === //

/// A single card within the [`ThemeStylesEditor`].
///
/// Composes three sections:
///
/// 1. **Header** -- key selector (select for built-ins, text for custom IDs)
///    and a remove button.
/// 2. **Style aliases** -- list of applied aliases with remove buttons + add.
/// 3. **Theme attributes** -- key/value rows with remove buttons + add.
#[component]
pub fn CssClassPartialsCard(
    input_diagram: Signal<InputDiagram<'static>>,
    target: ThemeStylesTarget,
    entry_index: usize,
    entry_key: String,
    style_aliases: Vec<String>,
    theme_attrs: Vec<ThemeAttrEntry>,
) -> Element {
    rsx! {
        div {
            class: CARD_CLASS,

            // === Header row: key + remove === //
            CssClassPartialsCardHeader {
                input_diagram,
                target: target.clone(),
                entry_key: entry_key.clone(),
            }

            // === Style aliases applied === //
            CssClassPartialsCardAliases {
                input_diagram,
                target: target.clone(),
                entry_key: entry_key.clone(),
                style_aliases,
            }

            // === Theme attributes (partials map) === //
            CssClassPartialsCardAttrs {
                input_diagram,
                target,
                entry_key,
                theme_attrs,
            }
        }
    }
}
