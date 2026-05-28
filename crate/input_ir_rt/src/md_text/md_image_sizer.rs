use base64::{prelude::BASE64_STANDARD, Engine};

use crate::md_text::md_blocks_parser::MdTokenItem;

/// Computes pixel dimensions for inline image tokens.
pub(crate) struct MdImageSizer;

impl MdImageSizer {
    /// Returns `(width, height)` in pixels for the given image token item,
    /// using the following priority order:
    ///
    /// 1. Explicit dimensions from an alt-text `{WxH}` annotation.
    /// 2. Intrinsic size decoded from a base64 PNG data URL.
    /// 3. Proportional scaling when only one dimension is known.
    /// 4. Fallback: `100.0 x 100.0`.
    ///
    /// For non-`Image` variants, returns `(0.0, 0.0)`.
    pub(crate) fn compute_size(item: &MdTokenItem) -> (f32, f32) {
        match item {
            MdTokenItem::Image {
                src,
                explicit_width,
                explicit_height,
                ..
            } => match (*explicit_width, *explicit_height) {
                (Some(w), Some(h)) => (w, h),
                (Some(w), None) => {
                    if let Some((iw, ih)) = Self::png_intrinsic_size(src) {
                        if iw > 0.0 {
                            (w, w * ih / iw)
                        } else {
                            (w, 100.0)
                        }
                    } else {
                        (w, 100.0)
                    }
                }
                (None, Some(h)) => {
                    if let Some((iw, ih)) = Self::png_intrinsic_size(src) {
                        if ih > 0.0 {
                            (h * iw / ih, h)
                        } else {
                            (100.0, h)
                        }
                    } else {
                        (100.0, h)
                    }
                }
                (None, None) => Self::png_intrinsic_size(src).unwrap_or((100.0, 100.0)),
            },
            MdTokenItem::Word { .. } | MdTokenItem::LineBreak => (0.0, 0.0),
        }
    }

    /// Attempts to read the intrinsic pixel dimensions from a base64 PNG data
    /// URL by decoding the IHDR chunk.
    ///
    /// Returns `None` if the URL is not a PNG data URL or decoding fails.
    fn png_intrinsic_size(src: &str) -> Option<(f32, f32)> {
        let data = src.strip_prefix("data:image/png;base64,")?;
        let bytes = BASE64_STANDARD.decode(data).ok()?;
        if bytes.len() < 24 {
            return None;
        }
        let width = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
        let height = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
        Some((width as f32, height as f32))
    }
}
