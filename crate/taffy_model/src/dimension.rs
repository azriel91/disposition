use serde::{Deserialize, Serialize};
use taffy::Size;

/// The width and height of a diagram.
///
/// These dimensions correspond to Tailwind CSS' [responsive breakpoints].
///
/// [responsive breakpoints]: https://tailwindcss.com/docs/responsive-design
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Dimension {
    /// Diagram should fit within 640x480.
    Sm,
    /// Diagram should fit within 768x512.
    Md,
    /// Diagram should fit within 1024x768.
    Lg,
    /// Diagram should fit within 1280x1024.
    Xl,
    /// Diagram should fit within 1536x1280.
    _2xl,
    /// Custom dimension with specified width and height.
    Custom { width: f32, height: f32 },
}

impl Dimension {
    pub fn width(self) -> f32 {
        match self {
            Dimension::Sm => 640.0,
            Dimension::Md => 768.0,
            Dimension::Lg => 1024.0,
            Dimension::Xl => 1280.0,
            Dimension::_2xl => 1536.0,
            Dimension::Custom { width, height: _ } => width,
        }
    }

    pub fn height(self) -> f32 {
        match self {
            Dimension::Sm => 480.0,
            Dimension::Md => 512.0,
            Dimension::Lg => 768.0,
            Dimension::Xl => 1024.0,
            Dimension::_2xl => 1280.0,
            Dimension::Custom { width: _, height } => height,
        }
    }

    /// Returns this dimension as a [`taffy::Size<f32>`]
    ///
    /// [`taffy::Size<f32>`]: taffy::Size
    pub fn size(self) -> Size<f32> {
        Size {
            width: self.width(),
            height: self.height(),
        }
    }
}

impl From<Dimension> for Size<f32> {
    fn from(value: Dimension) -> Self {
        value.size()
    }
}
