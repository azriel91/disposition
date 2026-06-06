use super::TailwindColorShade;

/// Pure computations for dark-mode shade derivation and stroke-style mapping.
///
/// These helpers are stateless and operate purely on shade / style strings, so
/// they are grouped here separately from the stateful
/// [`TailwindClassState`](super::TailwindClassState) class writers.
pub(crate) struct ShadeComputer;

impl ShadeComputer {
    /// Convert stroke style to stroke-dasharray value.
    ///
    /// Example valid `style` values: `"solid"`, `"dashed"`, `"dotted"`,
    /// `"dasharray:4 2"`.
    pub(crate) fn stroke_style_to_dasharray(style: &str) -> Option<&str> {
        match style {
            // `stroke-dasharray: none` in CSS produces a solid line.
            "solid" => Some("none"),
            "dashed" => Some("4"),
            "dotted" => Some("2"),
            s if s.starts_with("dasharray:") => Some(&s["dasharray:".len()..]),
            _ => None,
        }
    }

    /// Invert a tailwind shade number for dark mode.
    ///
    /// This is used for **text** colours where the dark-mode shade is the
    /// mirror image of the light-mode shade.
    ///
    /// Uses the following mapping:
    ///
    /// * `50` <-> `950`
    /// * `100` <-> `900`
    /// * `200` <-> `800`
    /// * `300` <-> `700`
    /// * `400` <-> `600`
    /// * `500` <-> `500`
    pub(crate) fn shade_inverted(shade: &str) -> &str {
        match shade {
            "50" => "950",
            "100" => "900",
            "200" => "800",
            "300" => "700",
            "400" => "600",
            "500" => "500",
            "600" => "400",
            "700" => "300",
            "800" => "200",
            "900" => "100",
            "950" => "50",
            other => other,
        }
    }

    /// Compute the dark-mode shade for a fill or stroke shade by shifting
    /// rather than inverting.
    ///
    /// The shift preserves the relative ordering of highlight-state shades so
    /// that, for example, `hover < normal < focus < active` in light mode is
    /// still `hover < normal < focus < active` in dark mode.
    ///
    /// # Shift direction
    ///
    /// The direction is determined by the `normal` shade of the group:
    ///
    /// * Normal shade `<= _400` -- the group is on the light end, so the
    ///   dark-mode shift goes **darker** (toward `_950`).
    /// * Normal shade `>= _600` -- the group is on the dark end, so the
    ///   dark-mode shift goes **lighter** (toward `_50`).
    /// * Normal shade `== _500` -- the tie-breaker examines the other shades in
    ///   the group: if they lean darker (majority index > 5), the dark-mode
    ///   shift goes lighter; if they lean lighter (majority index < 5), the
    ///   dark-mode shift goes darker. When exactly tied, the shift goes darker.
    ///
    /// # Parameters
    ///
    /// * `shade`: The shade string to shift, e.g. `"100"`, `"700"`.
    /// * `shade_normal`: The shade string for `HighlightState::Normal`.
    /// * `shade_hover`: The shade string for `HighlightState::Hover`.
    /// * `shade_focus`: The shade string for `HighlightState::Focus`.
    /// * `shade_active`: The shade string for `HighlightState::Active`.
    ///
    /// # Returns
    ///
    /// The shifted shade as a `&'static str`, or the original `shade` if it
    /// cannot be parsed as a known tailwind shade.
    pub(crate) fn shade_shifted<'a>(
        shade: &'a str,
        levels: u8,
        shade_normal: Option<&str>,
        shade_hover: Option<&str>,
        shade_focus: Option<&str>,
        shade_active: Option<&str>,
    ) -> &'a str {
        let Ok(shade_parsed) = shade.parse::<TailwindColorShade>() else {
            return shade;
        };

        let shift_darker =
            Self::shade_shift_is_darker(shade_normal, shade_hover, shade_focus, shade_active);

        let dark_shade = if shift_darker {
            shade_parsed.darker(levels)
        } else {
            shade_parsed.lighter(levels)
        };

        dark_shade.as_str()
    }

    /// Determine whether the dark-mode shift direction should go darker.
    ///
    /// Returns `true` when the shift should go darker (light shades in light
    /// mode become darker in dark mode), `false` when the shift should go
    /// lighter.
    ///
    /// # Parameters
    ///
    /// * `shade_normal`: The shade string for `HighlightState::Normal`.
    /// * `shade_hover`: The shade string for `HighlightState::Hover`.
    /// * `shade_focus`: The shade string for `HighlightState::Focus`.
    /// * `shade_active`: The shade string for `HighlightState::Active`.
    pub(crate) fn shade_shift_is_darker(
        shade_normal: Option<&str>,
        shade_hover: Option<&str>,
        shade_focus: Option<&str>,
        shade_active: Option<&str>,
    ) -> bool {
        let normal = shade_normal.and_then(|shade| shade.parse::<TailwindColorShade>().ok());

        match normal {
            Some(n) if n < TailwindColorShade::_500 => true,
            Some(n) if n > TailwindColorShade::_500 => false,
            Some(_) => {
                // Normal is exactly _500 -- look at the other shades.
                // If the majority leans dark (index > 5), shift lighter.
                // If the majority leans light (index < 5), shift darker.
                // Ties go darker.
                let mid = TailwindColorShade::_500.index();
                let lean = [shade_hover, shade_focus, shade_active]
                    .into_iter()
                    .flatten()
                    .filter_map(|shade_str| shade_str.parse::<TailwindColorShade>().ok())
                    .fold(0i32, |lean, shade| match shade.index().cmp(&mid) {
                        std::cmp::Ordering::Greater => lean + 1, // leans dark
                        std::cmp::Ordering::Less => lean - 1,    // leans light
                        std::cmp::Ordering::Equal => lean,
                    });

                // Shades leaning dark in light mode (positive lean) shift
                // lighter; leaning light or tied shifts darker.
                lean <= 0
            }
            // No normal shade available -- fall back to shifting darker.
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stroke_style_to_dasharray_maps_known_styles() {
        assert_eq!(Some("none"), ShadeComputer::stroke_style_to_dasharray("solid"));
        assert_eq!(Some("4"), ShadeComputer::stroke_style_to_dasharray("dashed"));
        assert_eq!(Some("2"), ShadeComputer::stroke_style_to_dasharray("dotted"));
        assert_eq!(
            Some("4 2"),
            ShadeComputer::stroke_style_to_dasharray("dasharray:4 2")
        );
        assert_eq!(None, ShadeComputer::stroke_style_to_dasharray("unknown"));
    }

    #[test]
    fn shade_inverted_mirrors_around_500() {
        assert_eq!("950", ShadeComputer::shade_inverted("50"));
        assert_eq!("500", ShadeComputer::shade_inverted("500"));
        assert_eq!("50", ShadeComputer::shade_inverted("950"));
        assert_eq!("300", ShadeComputer::shade_inverted("700"));
        // Unknown shades pass through unchanged.
        assert_eq!("abc", ShadeComputer::shade_inverted("abc"));
    }

    #[test]
    fn shade_shift_is_darker_light_normal_goes_darker() {
        assert!(ShadeComputer::shade_shift_is_darker(
            Some("100"),
            None,
            None,
            None
        ));
    }

    #[test]
    fn shade_shift_is_darker_dark_normal_goes_lighter() {
        assert!(!ShadeComputer::shade_shift_is_darker(
            Some("800"),
            None,
            None,
            None
        ));
    }

    #[test]
    fn shade_shift_is_darker_500_tie_breaks_on_other_shades() {
        // Other shades lean dark -> shift lighter (false).
        assert!(!ShadeComputer::shade_shift_is_darker(
            Some("500"),
            Some("700"),
            Some("800"),
            None
        ));
        // Other shades lean light -> shift darker (true).
        assert!(ShadeComputer::shade_shift_is_darker(
            Some("500"),
            Some("200"),
            Some("100"),
            None
        ));
        // Exactly tied -> shift darker (true).
        assert!(ShadeComputer::shade_shift_is_darker(
            Some("500"),
            Some("300"),
            Some("700"),
            None
        ));
    }

    #[test]
    fn shade_shift_is_darker_no_normal_defaults_darker() {
        assert!(ShadeComputer::shade_shift_is_darker(None, None, None, None));
    }

    #[test]
    fn shade_shifted_unparseable_returns_input() {
        assert_eq!(
            "weird",
            ShadeComputer::shade_shifted("weird", 2, Some("100"), None, None, None)
        );
    }
}
