use std::fmt::Write;

use disposition_ir_model::node::NodeShape;

/// Builds an SVG `<path>` `d` attribute string for a rectangle with optional
/// corner radii.
#[derive(Clone, Copy, Debug)]
pub struct SvgNodeRectPathBuilder;

impl SvgNodeRectPathBuilder {
    /// Builds an SVG path `d` attribute for a rectangle with optional corner
    /// radii.
    ///
    /// The path is constructed to draw a rectangle starting from just after
    /// the top-left corner (if rounded), proceeding clockwise:
    /// 1. Horizontal line to top-right corner
    /// 2. Arc for top-right corner (if radius > 0)
    /// 3. Vertical line to bottom-right corner
    /// 4. Arc for bottom-right corner (if radius > 0)
    /// 5. Horizontal line to bottom-left corner
    /// 6. Arc for bottom-left corner (if radius > 0)
    /// 7. Vertical line to top-left corner
    /// 8. Arc for top-left corner (if radius > 0)
    /// 9. Close path
    ///
    /// # Parameters
    ///
    /// * `width`: The width of the rectangle
    /// * `height`: The height of the rectangle
    /// * `node_shape`: The shape configuration containing corner radii
    pub fn build(width: f32, height: f32, node_shape: &NodeShape) -> String {
        let (r_tl, r_tr, r_bl, r_br) = match node_shape {
            NodeShape::Rect(rect) => (
                rect.radius_top_left,
                rect.radius_top_right,
                rect.radius_bottom_left,
                rect.radius_bottom_right,
            ),
            // Circle nodes still get a rectangular background path (made
            // invisible via wrapper_tailwind_classes); the actual circle is
            // rendered as a separate `<path>` element.
            //
            // Still use rounded corners for the wrapper node.
            NodeShape::Circle(_) => (4.0, 4.0, 4.0, 4.0),
        };

        let h = height;

        let mut d = String::with_capacity(128);

        // Move to start position (after top-left corner)
        write!(d, "M {r_tl} 0").unwrap();

        // Top edge: horizontal line to (width - r_tr, 0)
        write!(d, " H {}", width - r_tr).unwrap();

        // Top-right corner arc (if radius > 0)
        if r_tr > 0.0 {
            write!(d, " A {r_tr} {r_tr} 0 0 1 {width} {r_tr}").unwrap();
        }

        // Right edge: vertical line to (width, h - r_br)
        write!(d, " V {}", h - r_br).unwrap();

        // Bottom-right corner arc (if radius > 0)
        if r_br > 0.0 {
            write!(d, " A {r_br} {r_br} 0 0 1 {} {h}", width - r_br).unwrap();
        }

        // Bottom edge: horizontal line to (r_bl, h)
        write!(d, " H {r_bl}").unwrap();

        // Bottom-left corner arc (if radius > 0)
        if r_bl > 0.0 {
            write!(d, " A {r_bl} {r_bl} 0 0 1 0 {}", h - r_bl).unwrap();
        }

        // Left edge: vertical line to (0, r_tl)
        write!(d, " V {r_tl}").unwrap();

        // Top-left corner arc (if radius > 0)
        if r_tl > 0.0 {
            write!(d, " A {r_tl} {r_tl} 0 0 1 {r_tl} 0").unwrap();
        }

        // Close the path
        d.push_str(" Z");

        d
    }
}
