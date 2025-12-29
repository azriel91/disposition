use cosmic_text::{Align, Attrs, Buffer, FontSystem, Metrics, Shaping};
use taffy::{self, AvailableSpace};

/// Wraps a `cosmic_text::Buffer` and provides the `measure` function to measure
/// the size needed to render text within a `taffy` leaf node.
///
/// An optimization we could do is allocate a single `Buffer` for all the
/// generated taffy trees, and pass it in during `measure` to compute the size.
#[derive(Clone, Debug)]
pub struct CosmicTextContext {
    buffer: Buffer,
}

impl CosmicTextContext {
    /// Returns a new `CosmicTextContext`.
    ///
    /// # Parameters
    ///
    /// * `metrics`: Font size and line height.
    /// * `font_system`: Font database, locale, and cache.
    /// * `text`: The text to measure.
    /// * `attrs`: Color, style, weight, etc.
    pub fn new(metrics: Metrics, font_system: &mut FontSystem, attrs: &Attrs, text: &str) -> Self {
        let mut buffer = Buffer::new_empty(metrics);
        buffer.set_size(font_system, None, None);
        buffer.set_text(
            font_system,
            text,
            attrs,
            Shaping::Advanced,
            Some(Align::Left),
        );
        Self { buffer }
    }

    pub fn measure(
        &mut self,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        font_system: &mut FontSystem,
    ) -> taffy::Size<f32> {
        // Set width constraint
        let width_constraint = known_dimensions.width.or(match available_space.width {
            AvailableSpace::MinContent => Some(0.0),
            AvailableSpace::MaxContent => None,
            AvailableSpace::Definite(width) => Some(width),
        });
        self.buffer.set_size(font_system, width_constraint, None);

        // Compute layout
        self.buffer.shape_until_scroll(font_system, false);

        // Determine measured size of text
        let (width, total_lines) = self
            .buffer
            .layout_runs()
            .fold((0.0, 0usize), |(width, total_lines), run| {
                (run.line_w.max(width), total_lines + 1)
            });
        let height = total_lines as f32 * self.buffer.metrics().line_height;

        taffy::Size { width, height }
    }
}
