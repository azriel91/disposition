//! Asserts the renderer's Tailwind tables stay in sync with the shared value
//! lists in `disposition_model_common::theme` (used by the LSP for value
//! completions).

use disposition::model_common::theme::{TAILWIND_COLOR_NAMES, TAILWIND_COLOR_SHADES};
use disposition_input_ir_rt::{TailwindColorShade, TAILWIND_COLORS};

#[test]
fn color_names_match_shared_list() {
    let names = TAILWIND_COLORS
        .iter()
        .map(|tailwind_color| tailwind_color.name)
        .collect::<Vec<&str>>();

    assert_eq!(TAILWIND_COLOR_NAMES, names.as_slice());
}

#[test]
fn shades_match_shared_list() {
    // Every shared shade parses to a renderer shade and round-trips back.
    for shade in TAILWIND_COLOR_SHADES {
        let parsed = shade
            .parse::<TailwindColorShade>()
            .unwrap_or_else(|_| panic!("shade `{shade}` is not a known `TailwindColorShade`"));
        assert_eq!(*shade, parsed.as_str());
    }

    // Every renderer color provides exactly the shared shades, in order.
    for tailwind_color in TAILWIND_COLORS {
        let shades = tailwind_color
            .shades
            .iter()
            .map(|(shade, _oklch)| *shade)
            .collect::<Vec<&str>>();

        assert_eq!(
            TAILWIND_COLOR_SHADES,
            shades.as_slice(),
            "color `{}` shades differ from the shared list",
            tailwind_color.name
        );
    }
}
