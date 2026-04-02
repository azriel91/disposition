//! Tailwind CSS color shade.
//!
//! This module provides the [`TailwindColorShade`] enum, representing the shade
//! component of a Tailwind CSS color class. Shades range from `_50` (lightest)
//! to `_950` (darkest) and support relative navigation via
//! [`TailwindColorShade::darker`] and [`TailwindColorShade::lighter`].

// === Constants === //

/// All tailwind color shades ordered from lightest to darkest.
const ALL_SHADES: [TailwindColorShade; 11] = [
    TailwindColorShade::_50,
    TailwindColorShade::_100,
    TailwindColorShade::_200,
    TailwindColorShade::_300,
    TailwindColorShade::_400,
    TailwindColorShade::_500,
    TailwindColorShade::_600,
    TailwindColorShade::_700,
    TailwindColorShade::_800,
    TailwindColorShade::_900,
    TailwindColorShade::_950,
];

// === Types === //

/// A Tailwind CSS color shade value.
///
/// Represents the shade component of a Tailwind color class, ranging from
/// `_50` (lightest) to `_950` (darkest). Shades are ordered from light to
/// dark and support relative navigation via [`TailwindColorShade::darker`]
/// and [`TailwindColorShade::lighter`].
///
/// # Examples
///
/// Valid shades: `"50"`, `"100"`, `"200"`, `"300"`, `"400"`, `"500"`,
/// `"600"`, `"700"`, `"800"`, `"900"`, `"950"`.
///
/// ```rust,ignore
/// let shade = TailwindColorShade::_100;
/// assert_eq!(shade.darker(2), TailwindColorShade::_300);
/// assert_eq!(shade.lighter(1), TailwindColorShade::_50);
/// assert_eq!(shade.as_str(), "100");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum TailwindColorShade {
    /// Shade 50 -- lightest.
    _50,
    /// Shade 100.
    _100,
    /// Shade 200.
    _200,
    /// Shade 300.
    _300,
    /// Shade 400.
    _400,
    /// Shade 500.
    _500,
    /// Shade 600.
    _600,
    /// Shade 700.
    _700,
    /// Shade 800.
    _800,
    /// Shade 900.
    _900,
    /// Shade 950 -- darkest.
    _950,
}

impl TailwindColorShade {
    /// Parses a shade string into a `TailwindColorShade`.
    ///
    /// Returns `None` if the string does not match a known shade.
    ///
    /// # Parameters
    ///
    /// * `s`: the shade string, e.g. `"50"`, `"100"`, .. `"950"`.
    ///
    /// # Examples
    ///
    /// Valid inputs: `"50"`, `"100"`, `"200"`, `"300"`, `"400"`, `"500"`,
    /// `"600"`, `"700"`, `"800"`, `"900"`, `"950"`.
    ///
    /// ```rust,ignore
    /// assert_eq!(
    ///     TailwindColorShade::from_str("500"),
    ///     Some(TailwindColorShade::_500),
    /// );
    /// assert_eq!(TailwindColorShade::from_str("999"), None);
    /// ```
    pub(crate) fn from_str(s: &str) -> Option<TailwindColorShade> {
        match s {
            "50" => Some(TailwindColorShade::_50),
            "100" => Some(TailwindColorShade::_100),
            "200" => Some(TailwindColorShade::_200),
            "300" => Some(TailwindColorShade::_300),
            "400" => Some(TailwindColorShade::_400),
            "500" => Some(TailwindColorShade::_500),
            "600" => Some(TailwindColorShade::_600),
            "700" => Some(TailwindColorShade::_700),
            "800" => Some(TailwindColorShade::_800),
            "900" => Some(TailwindColorShade::_900),
            "950" => Some(TailwindColorShade::_950),
            _ => None,
        }
    }

    /// Returns the string representation of this shade.
    ///
    /// # Examples
    ///
    /// Return values: `"50"`, `"100"`, `"200"`, `"300"`, `"400"`, `"500"`,
    /// `"600"`, `"700"`, `"800"`, `"900"`, `"950"`.
    ///
    /// ```rust,ignore
    /// assert_eq!(TailwindColorShade::_50.as_str(), "50");
    /// assert_eq!(TailwindColorShade::_950.as_str(), "950");
    /// ```
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            TailwindColorShade::_50 => "50",
            TailwindColorShade::_100 => "100",
            TailwindColorShade::_200 => "200",
            TailwindColorShade::_300 => "300",
            TailwindColorShade::_400 => "400",
            TailwindColorShade::_500 => "500",
            TailwindColorShade::_600 => "600",
            TailwindColorShade::_700 => "700",
            TailwindColorShade::_800 => "800",
            TailwindColorShade::_900 => "900",
            TailwindColorShade::_950 => "950",
        }
    }

    /// Returns the zero-based index of this shade in the light-to-dark
    /// ordering.
    ///
    /// `_50` is index 0, `_100` is index 1, .. `_950` is index 10.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// assert_eq!(TailwindColorShade::_50.index(), 0);
    /// assert_eq!(TailwindColorShade::_500.index(), 5);
    /// assert_eq!(TailwindColorShade::_950.index(), 10);
    /// ```
    pub(crate) fn index(self) -> usize {
        match self {
            TailwindColorShade::_50 => 0,
            TailwindColorShade::_100 => 1,
            TailwindColorShade::_200 => 2,
            TailwindColorShade::_300 => 3,
            TailwindColorShade::_400 => 4,
            TailwindColorShade::_500 => 5,
            TailwindColorShade::_600 => 6,
            TailwindColorShade::_700 => 7,
            TailwindColorShade::_800 => 8,
            TailwindColorShade::_900 => 9,
            TailwindColorShade::_950 => 10,
        }
    }

    /// Returns the shade corresponding to the given index, clamped to the
    /// valid range `0..=10`.
    ///
    /// Index 0 corresponds to `_50`, index 10 corresponds to `_950`. Values
    /// above 10 are clamped to `_950`.
    ///
    /// # Parameters
    ///
    /// * `index`: zero-based shade index, e.g. `0` for `_50`, `10` for `_950`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// assert_eq!(TailwindColorShade::from_index(0), TailwindColorShade::_50);
    /// assert_eq!(TailwindColorShade::from_index(5), TailwindColorShade::_500);
    /// assert_eq!(TailwindColorShade::from_index(99), TailwindColorShade::_950);
    /// ```
    pub(crate) fn from_index(index: usize) -> TailwindColorShade {
        let clamped = if index >= ALL_SHADES.len() {
            ALL_SHADES.len() - 1
        } else {
            index
        };
        ALL_SHADES[clamped]
    }

    /// Returns the shade `levels` steps darker (toward `_950`), clamping at
    /// `_950`.
    ///
    /// Each level moves one position in the shade ordering. For example,
    /// moving 1 level darker from `_100` yields `_200`.
    ///
    /// # Parameters
    ///
    /// * `levels`: number of shade steps toward darker, e.g. `1`, `2`, `5`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// assert_eq!(TailwindColorShade::_100.darker(2), TailwindColorShade::_300);
    /// assert_eq!(TailwindColorShade::_900.darker(5), TailwindColorShade::_950);
    /// assert_eq!(TailwindColorShade::_50.darker(0), TailwindColorShade::_50);
    /// ```
    pub(crate) fn darker(self, levels: u8) -> TailwindColorShade {
        let new_index = self.index().saturating_add(levels as usize);
        TailwindColorShade::from_index(new_index)
    }

    /// Returns the shade `levels` steps lighter (toward `_50`), clamping at
    /// `_50`.
    ///
    /// Each level moves one position in the shade ordering. For example,
    /// moving 1 level lighter from `_200` yields `_100`.
    ///
    /// # Parameters
    ///
    /// * `levels`: number of shade steps toward lighter, e.g. `1`, `2`, `5`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// assert_eq!(TailwindColorShade::_300.lighter(2), TailwindColorShade::_100);
    /// assert_eq!(TailwindColorShade::_100.lighter(5), TailwindColorShade::_50);
    /// assert_eq!(TailwindColorShade::_950.lighter(0), TailwindColorShade::_950);
    /// ```
    pub(crate) fn lighter(self, levels: u8) -> TailwindColorShade {
        let new_index = self.index().saturating_sub(levels as usize);
        TailwindColorShade::from_index(new_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_parses_all_valid_shades() {
        assert_eq!(
            TailwindColorShade::from_str("50"),
            Some(TailwindColorShade::_50)
        );
        assert_eq!(
            TailwindColorShade::from_str("100"),
            Some(TailwindColorShade::_100)
        );
        assert_eq!(
            TailwindColorShade::from_str("200"),
            Some(TailwindColorShade::_200)
        );
        assert_eq!(
            TailwindColorShade::from_str("300"),
            Some(TailwindColorShade::_300)
        );
        assert_eq!(
            TailwindColorShade::from_str("400"),
            Some(TailwindColorShade::_400)
        );
        assert_eq!(
            TailwindColorShade::from_str("500"),
            Some(TailwindColorShade::_500)
        );
        assert_eq!(
            TailwindColorShade::from_str("600"),
            Some(TailwindColorShade::_600)
        );
        assert_eq!(
            TailwindColorShade::from_str("700"),
            Some(TailwindColorShade::_700)
        );
        assert_eq!(
            TailwindColorShade::from_str("800"),
            Some(TailwindColorShade::_800)
        );
        assert_eq!(
            TailwindColorShade::from_str("900"),
            Some(TailwindColorShade::_900)
        );
        assert_eq!(
            TailwindColorShade::from_str("950"),
            Some(TailwindColorShade::_950)
        );
    }

    #[test]
    fn from_str_returns_none_for_invalid_input() {
        assert_eq!(TailwindColorShade::from_str("0"), None);
        assert_eq!(TailwindColorShade::from_str("999"), None);
        assert_eq!(TailwindColorShade::from_str(""), None);
        assert_eq!(TailwindColorShade::from_str("abc"), None);
    }

    #[test]
    fn as_str_returns_expected_values() {
        assert_eq!(TailwindColorShade::_50.as_str(), "50");
        assert_eq!(TailwindColorShade::_500.as_str(), "500");
        assert_eq!(TailwindColorShade::_950.as_str(), "950");
    }

    #[test]
    fn from_str_roundtrips_with_as_str() {
        for shade in ALL_SHADES {
            let shade_str = shade.as_str();
            assert_eq!(TailwindColorShade::from_str(shade_str), Some(shade));
        }
    }

    #[test]
    fn index_returns_sequential_values() {
        for (expected_index, shade) in ALL_SHADES.iter().enumerate() {
            assert_eq!(shade.index(), expected_index);
        }
    }

    #[test]
    fn from_index_returns_correct_shade() {
        for (index, expected_shade) in ALL_SHADES.iter().enumerate() {
            assert_eq!(TailwindColorShade::from_index(index), *expected_shade);
        }
    }

    #[test]
    fn from_index_clamps_at_maximum() {
        assert_eq!(TailwindColorShade::from_index(11), TailwindColorShade::_950);
        assert_eq!(
            TailwindColorShade::from_index(100),
            TailwindColorShade::_950
        );
        assert_eq!(
            TailwindColorShade::from_index(usize::MAX),
            TailwindColorShade::_950
        );
    }

    #[test]
    fn darker_moves_toward_950() {
        assert_eq!(TailwindColorShade::_50.darker(1), TailwindColorShade::_100);
        assert_eq!(TailwindColorShade::_100.darker(2), TailwindColorShade::_300);
        assert_eq!(TailwindColorShade::_50.darker(10), TailwindColorShade::_950);
    }

    #[test]
    fn darker_clamps_at_950() {
        assert_eq!(TailwindColorShade::_900.darker(5), TailwindColorShade::_950);
        assert_eq!(TailwindColorShade::_950.darker(1), TailwindColorShade::_950);
        assert_eq!(
            TailwindColorShade::_50.darker(255),
            TailwindColorShade::_950
        );
    }

    #[test]
    fn darker_zero_is_identity() {
        for shade in ALL_SHADES {
            assert_eq!(shade.darker(0), shade);
        }
    }

    #[test]
    fn lighter_moves_toward_50() {
        assert_eq!(TailwindColorShade::_100.lighter(1), TailwindColorShade::_50);
        assert_eq!(
            TailwindColorShade::_300.lighter(2),
            TailwindColorShade::_100
        );
        assert_eq!(
            TailwindColorShade::_950.lighter(10),
            TailwindColorShade::_50
        );
    }

    #[test]
    fn lighter_clamps_at_50() {
        assert_eq!(TailwindColorShade::_100.lighter(5), TailwindColorShade::_50);
        assert_eq!(TailwindColorShade::_50.lighter(1), TailwindColorShade::_50);
        assert_eq!(
            TailwindColorShade::_950.lighter(255),
            TailwindColorShade::_50
        );
    }

    #[test]
    fn lighter_zero_is_identity() {
        for shade in ALL_SHADES {
            assert_eq!(shade.lighter(0), shade);
        }
    }

    #[test]
    fn darker_then_lighter_roundtrips() {
        assert_eq!(
            TailwindColorShade::_500.darker(3).lighter(3),
            TailwindColorShade::_500
        );
        assert_eq!(
            TailwindColorShade::_50.darker(5).lighter(5),
            TailwindColorShade::_50
        );
    }

    #[test]
    fn ordering_is_light_to_dark() {
        for window in ALL_SHADES.windows(2) {
            assert!(window[0] < window[1]);
        }
    }
}
