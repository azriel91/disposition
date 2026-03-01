use disposition_input_model::{
    entity::EntityTypes,
    theme::{
        CssClassPartials, IdOrDefaults, StyleAliases, ThemeAttr, ThemeDefault, ThemeTypesStyles,
    },
};
use disposition_ir_model::{
    entity::EntityTypeId,
    node::{NodeShapeCircle, NodeShapeRect},
};
use disposition_model_common::Id;

/// Resolves theme attributes (padding, margin, gap, radius) from theme
/// configuration.
///
/// Resolution follows a priority order:
///
/// 1. `NodeDefaults` from `theme_default` (lowest priority)
/// 2. `EntityType`s applied to the node (in order, later overrides earlier)
/// 3. The `NodeId` itself from `theme_default` (highest priority)
///
/// Within each level, `StyleAlias`es are applied first, then direct
/// attributes.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ThemeAttrResolver;

impl ThemeAttrResolver {
    /// Resolves a theme attribute value by traversing theme sources in priority
    /// order.
    ///
    /// # Parameters
    ///
    /// * `node_id`: The entity ID to resolve attributes for.
    /// * `entity_types`: Entity types for lookup.
    /// * `theme_default`: The default theme configuration.
    /// * `theme_types_styles`: Styles for each entity type.
    /// * `state`: Mutable state that accumulates resolved values.
    /// * `apply_from_partials`: Closure that extracts values from
    ///   `CssClassPartials` and applies them to state, considering style
    ///   aliases.
    /// * `finalize`: Closure that converts the accumulated state into the final
    ///   result with defaults.
    pub(crate) fn resolve_theme_attr<'id, State, Result>(
        node_id: &Id<'id>,
        entity_types: &EntityTypes<'id>,
        theme_default: &ThemeDefault<'id>,
        theme_types_styles: &ThemeTypesStyles<'id>,
        state: &mut State,
        apply_from_partials: impl Fn(&CssClassPartials<'id>, &StyleAliases<'id>, &mut State),
        finalize: impl FnOnce(&State) -> Result,
    ) -> Result {
        // 1. Start with NodeDefaults (lowest priority)
        if let Some(node_defaults_partials) =
            theme_default.base_styles.get(&IdOrDefaults::NodeDefaults)
        {
            apply_from_partials(node_defaults_partials, &theme_default.style_aliases, state);
        }

        // 2. Apply EntityTypes in order (later types override earlier ones)
        if let Some(types) = entity_types.get(node_id) {
            types
                .iter()
                .filter_map(|entity_type| {
                    let type_id = EntityTypeId::from(entity_type.clone().into_id());
                    theme_types_styles
                        .get(&type_id)
                        .and_then(|type_styles| type_styles.get(&IdOrDefaults::NodeDefaults))
                })
                .for_each(|type_partials| {
                    apply_from_partials(type_partials, &theme_default.style_aliases, state);
                });
        }

        // 3. Apply node ID itself (highest priority)
        if let Some(node_partials) = theme_default
            .base_styles
            .get(&IdOrDefaults::Id(node_id.clone()))
        {
            apply_from_partials(node_partials, &theme_default.style_aliases, state);
        }

        finalize(state)
    }

    // === Padding === //

    pub(crate) fn resolve_padding<'id>(
        node_id: Option<&Id<'id>>,
        entity_types: &EntityTypes<'id>,
        theme_default: &ThemeDefault<'id>,
        theme_types_styles: &ThemeTypesStyles<'id>,
    ) -> (f32, f32, f32, f32) {
        let mut state = (None, None, None, None);

        if let Some(id) = node_id {
            Self::resolve_theme_attr(
                id,
                entity_types,
                theme_default,
                theme_types_styles,
                &mut state,
                Self::apply_padding_from_partials,
                |state| {
                    (
                        state.0.unwrap_or(0.0),
                        state.1.unwrap_or(0.0),
                        state.2.unwrap_or(0.0),
                        state.3.unwrap_or(0.0),
                    )
                },
            )
        } else {
            (0.0, 0.0, 0.0, 0.0)
        }
    }

    /// Apply padding values from CssClassPartials, checking both direct
    /// attributes and style aliases.
    fn apply_padding_from_partials<'id>(
        partials: &CssClassPartials<'id>,
        style_aliases: &StyleAliases<'id>,
        state: &mut (Option<f32>, Option<f32>, Option<f32>, Option<f32>),
    ) {
        // First, check style_aliases_applied (lower priority within this partials)
        partials
            .style_aliases_applied()
            .iter()
            .filter_map(|alias| style_aliases.get(alias))
            .for_each(|alias_partials| Self::extract_padding_from_map(alias_partials, state));

        // Then, check direct attributes (higher priority within this partials)
        Self::extract_padding_from_map(partials, state);
    }

    /// Extract padding values from a map of ThemeAttr to String.
    fn extract_padding_from_map<'id>(
        partials: &CssClassPartials<'id>,
        state: &mut (Option<f32>, Option<f32>, Option<f32>, Option<f32>),
    ) {
        let (padding_top, padding_right, padding_bottom, padding_left) = state;

        // Check compound Padding first (applies to all sides)
        if let Some(value) = partials.get(&ThemeAttr::Padding)
            && let Ok(v) = value.parse::<f32>()
        {
            *padding_top = Some(v);
            *padding_right = Some(v);
            *padding_bottom = Some(v);
            *padding_left = Some(v);
        }

        // Check PaddingX (horizontal) -- overrides Padding for left/right
        if let Some(value) = partials.get(&ThemeAttr::PaddingX)
            && let Ok(v) = value.parse::<f32>()
        {
            *padding_left = Some(v);
            *padding_right = Some(v);
        }

        // Check PaddingY (vertical) -- overrides Padding for top/bottom
        if let Some(value) = partials.get(&ThemeAttr::PaddingY)
            && let Ok(v) = value.parse::<f32>()
        {
            *padding_top = Some(v);
            *padding_bottom = Some(v);
        }

        // Check specific padding attributes (highest specificity)
        if let Some(value) = partials.get(&ThemeAttr::PaddingTop)
            && let Ok(v) = value.parse::<f32>()
        {
            *padding_top = Some(v);
        }
        if let Some(value) = partials.get(&ThemeAttr::PaddingRight)
            && let Ok(v) = value.parse::<f32>()
        {
            *padding_right = Some(v);
        }
        if let Some(value) = partials.get(&ThemeAttr::PaddingBottom)
            && let Ok(v) = value.parse::<f32>()
        {
            *padding_bottom = Some(v);
        }
        if let Some(value) = partials.get(&ThemeAttr::PaddingLeft)
            && let Ok(v) = value.parse::<f32>()
        {
            *padding_left = Some(v);
        }
    }

    // === Margin === //

    pub(crate) fn resolve_margin<'id>(
        node_id: Option<&Id<'id>>,
        entity_types: &EntityTypes<'id>,
        theme_default: &ThemeDefault<'id>,
        theme_types_styles: &ThemeTypesStyles<'id>,
    ) -> (f32, f32, f32, f32) {
        let mut state = (None, None, None, None);

        if let Some(id) = node_id {
            Self::resolve_theme_attr(
                id,
                entity_types,
                theme_default,
                theme_types_styles,
                &mut state,
                Self::apply_margin_from_partials,
                |state| {
                    (
                        state.0.unwrap_or(0.0),
                        state.1.unwrap_or(0.0),
                        state.2.unwrap_or(0.0),
                        state.3.unwrap_or(0.0),
                    )
                },
            )
        } else {
            (0.0, 0.0, 0.0, 0.0)
        }
    }

    /// Apply margin values from CssClassPartials, checking both direct
    /// attributes and style aliases.
    fn apply_margin_from_partials<'id>(
        partials: &CssClassPartials<'id>,
        style_aliases: &StyleAliases<'id>,
        state: &mut (Option<f32>, Option<f32>, Option<f32>, Option<f32>),
    ) {
        // First, check style_aliases_applied (lower priority within this partials)
        partials
            .style_aliases_applied()
            .iter()
            .filter_map(|alias| style_aliases.get(alias))
            .for_each(|alias_partials| Self::extract_margin_from_map(alias_partials, state));

        // Then, check direct attributes (higher priority within this partials)
        Self::extract_margin_from_map(partials, state);
    }

    /// Extract margin values from a map of ThemeAttr to String.
    fn extract_margin_from_map<'id>(
        partials: &CssClassPartials<'id>,
        state: &mut (Option<f32>, Option<f32>, Option<f32>, Option<f32>),
    ) {
        let (margin_top, margin_right, margin_bottom, margin_left) = state;

        // Check compound Margin first (applies to all sides)
        if let Some(value) = partials.get(&ThemeAttr::Margin)
            && let Ok(v) = value.parse::<f32>()
        {
            *margin_top = Some(v);
            *margin_right = Some(v);
            *margin_bottom = Some(v);
            *margin_left = Some(v);
        }

        // Check MarginX (horizontal) -- overrides Margin for left/right
        if let Some(value) = partials.get(&ThemeAttr::MarginX)
            && let Ok(v) = value.parse::<f32>()
        {
            *margin_left = Some(v);
            *margin_right = Some(v);
        }

        // Check MarginY (vertical) -- overrides Margin for top/bottom
        if let Some(value) = partials.get(&ThemeAttr::MarginY)
            && let Ok(v) = value.parse::<f32>()
        {
            *margin_top = Some(v);
            *margin_bottom = Some(v);
        }

        // Check specific margin attributes (highest specificity)
        if let Some(value) = partials.get(&ThemeAttr::MarginTop)
            && let Ok(v) = value.parse::<f32>()
        {
            *margin_top = Some(v);
        }
        if let Some(value) = partials.get(&ThemeAttr::MarginRight)
            && let Ok(v) = value.parse::<f32>()
        {
            *margin_right = Some(v);
        }
        if let Some(value) = partials.get(&ThemeAttr::MarginBottom)
            && let Ok(v) = value.parse::<f32>()
        {
            *margin_bottom = Some(v);
        }
        if let Some(value) = partials.get(&ThemeAttr::MarginLeft)
            && let Ok(v) = value.parse::<f32>()
        {
            *margin_left = Some(v);
        }
    }

    // === Gap === //

    pub(crate) fn resolve_gap<'id>(
        node_id: Option<&Id<'id>>,
        entity_types: &EntityTypes<'id>,
        theme_default: &ThemeDefault<'id>,
        theme_types_styles: &ThemeTypesStyles<'id>,
    ) -> f32 {
        let mut state = None;

        if let Some(id) = node_id {
            Self::resolve_theme_attr(
                id,
                entity_types,
                theme_default,
                theme_types_styles,
                &mut state,
                Self::apply_gap_from_partials,
                |state| state.unwrap_or(0.0),
            )
        } else {
            0.0
        }
    }

    /// Apply gap value from CssClassPartials, checking both direct
    /// and style aliases.
    fn apply_gap_from_partials<'id>(
        partials: &CssClassPartials<'id>,
        style_aliases: &StyleAliases<'id>,
        state: &mut Option<f32>,
    ) {
        // First, check style_aliases_applied (lower priority within this partials)
        partials
            .style_aliases_applied()
            .iter()
            .filter_map(|alias| style_aliases.get(alias))
            .filter_map(|alias_partials| alias_partials.get(&ThemeAttr::Gap))
            .filter_map(|value| value.parse::<f32>().ok())
            .for_each(|v| *state = Some(v));

        // Then, check direct attribute (higher priority within this partials)
        if let Some(value) = partials.get(&ThemeAttr::Gap)
            && let Ok(v) = value.parse::<f32>()
        {
            *state = Some(v);
        }
    }

    // === Circle Radius === //

    /// Resolve the circle radius for a node from the theme.
    ///
    /// Returns `Some(radius)` if a `ThemeAttr::CircleRadius` is configured
    /// for this node, or `None` if the node should use a rectangular shape.
    pub(crate) fn resolve_circle_radius<'id>(
        node_id: Option<&Id<'id>>,
        entity_types: &EntityTypes<'id>,
        theme_default: &ThemeDefault<'id>,
        theme_types_styles: &ThemeTypesStyles<'id>,
    ) -> Option<f32> {
        let mut state: Option<f32> = None;

        if let Some(id) = node_id {
            Self::resolve_theme_attr(
                id,
                entity_types,
                theme_default,
                theme_types_styles,
                &mut state,
                Self::apply_circle_radius_from_partials,
                |state| *state,
            )
        } else {
            None
        }
    }

    /// Apply circle radius from `CssClassPartials`, checking both direct
    /// attributes and style aliases.
    fn apply_circle_radius_from_partials<'id>(
        partials: &CssClassPartials<'id>,
        style_aliases: &StyleAliases<'id>,
        state: &mut Option<f32>,
    ) {
        // First, check style_aliases_applied (lower priority within this partials)
        partials
            .style_aliases_applied()
            .iter()
            .filter_map(|alias| style_aliases.get(alias))
            .for_each(|alias_partials| Self::extract_circle_radius_from_map(alias_partials, state));

        // Then, check direct attributes (higher priority within this partials)
        Self::extract_circle_radius_from_map(partials, state);
    }

    /// Extract circle radius value from a map of `ThemeAttr` to `String`.
    fn extract_circle_radius_from_map<'id>(
        partials: &CssClassPartials<'id>,
        state: &mut Option<f32>,
    ) {
        if let Some(value) = partials.get(&ThemeAttr::CircleRadius)
            && let Ok(v) = value.parse::<f32>()
        {
            *state = Some(v);
        }
    }

    // === Rect Radius === //

    /// Resolve corner radius values for a node from the theme.
    pub(crate) fn resolve_rect_radius<'id>(
        node_id: Option<&Id<'id>>,
        entity_types: &EntityTypes<'id>,
        theme_default: &ThemeDefault<'id>,
        theme_types_styles: &ThemeTypesStyles<'id>,
    ) -> (f32, f32, f32, f32) {
        let mut state = (None, None, None, None);

        if let Some(id) = node_id {
            Self::resolve_theme_attr(
                id,
                entity_types,
                theme_default,
                theme_types_styles,
                &mut state,
                Self::apply_radius_from_partials,
                |state| {
                    (
                        state.0.unwrap_or(0.0),
                        state.1.unwrap_or(0.0),
                        state.2.unwrap_or(0.0),
                        state.3.unwrap_or(0.0),
                    )
                },
            )
        } else {
            (0.0, 0.0, 0.0, 0.0)
        }
    }

    /// Apply radius values from CssClassPartials, checking both direct
    /// attributes and style aliases.
    fn apply_radius_from_partials<'id>(
        partials: &CssClassPartials<'id>,
        style_aliases: &StyleAliases<'id>,
        state: &mut (Option<f32>, Option<f32>, Option<f32>, Option<f32>),
    ) {
        // First, check style_aliases_applied (lower priority within this partials)
        partials
            .style_aliases_applied()
            .iter()
            .filter_map(|alias| style_aliases.get(alias))
            .for_each(|alias_partials| Self::extract_radius_from_map(alias_partials, state));

        // Then, check direct attributes (higher priority within this partials)
        Self::extract_radius_from_map(partials, state);
    }

    /// Extract radius values from a map of ThemeAttr to String.
    fn extract_radius_from_map<'id>(
        partials: &CssClassPartials<'id>,
        state: &mut (Option<f32>, Option<f32>, Option<f32>, Option<f32>),
    ) {
        let (radius_top_left, radius_top_right, radius_bottom_left, radius_bottom_right) = state;

        // Check specific radius attributes
        if let Some(value) = partials.get(&ThemeAttr::RadiusTopLeft)
            && let Ok(v) = value.parse::<f32>()
        {
            *radius_top_left = Some(v);
        }
        if let Some(value) = partials.get(&ThemeAttr::RadiusTopRight)
            && let Ok(v) = value.parse::<f32>()
        {
            *radius_top_right = Some(v);
        }
        if let Some(value) = partials.get(&ThemeAttr::RadiusBottomLeft)
            && let Ok(v) = value.parse::<f32>()
        {
            *radius_bottom_left = Some(v);
        }
        if let Some(value) = partials.get(&ThemeAttr::RadiusBottomRight)
            && let Ok(v) = value.parse::<f32>()
        {
            *radius_bottom_right = Some(v);
        }
    }

    // === Node Shape Resolution === //

    /// Resolve the node shape for a node from the theme.
    ///
    /// Returns a `NodeShapeCircle` if `CircleRadius` is configured, otherwise
    /// returns a `NodeShapeRect` with corner radii from the theme.
    pub(crate) fn resolve_node_shape<'id>(
        node_id: &Id<'id>,
        entity_types: &EntityTypes<'id>,
        theme_default: &ThemeDefault<'id>,
        theme_types_styles: &ThemeTypesStyles<'id>,
    ) -> disposition_ir_model::node::NodeShape {
        // First, check if this node has a circle radius configured.
        let circle_radius = Self::resolve_circle_radius(
            Some(node_id),
            entity_types,
            theme_default,
            theme_types_styles,
        );

        if let Some(radius) = circle_radius {
            disposition_ir_model::node::NodeShape::Circle(NodeShapeCircle::with_radius(radius))
        } else {
            let (radius_top_left, radius_top_right, radius_bottom_left, radius_bottom_right) =
                Self::resolve_rect_radius(
                    Some(node_id),
                    entity_types,
                    theme_default,
                    theme_types_styles,
                );

            disposition_ir_model::node::NodeShape::Rect(NodeShapeRect {
                radius_top_left,
                radius_top_right,
                radius_bottom_left,
                radius_bottom_right,
            })
        }
    }
}
