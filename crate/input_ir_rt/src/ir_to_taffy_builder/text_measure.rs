use unicode_segmentation::UnicodeSegmentation;

/// Monospace character width as a ratio of font size.
/// For Noto Sans Mono at 11px, the character width is approximately 6.6px (0.6
/// * 11).
pub(crate) const MONOSPACE_CHAR_WIDTH_RATIO: f32 = 0.6;
const EMOJI_CHAR_WIDTH: f32 = 2.29;

/// Compute text dimensions using simple monospace character width calculation.
/// Returns (max_line_width, line_count).
pub(crate) fn compute_text_dimensions(
    text: &str,
    char_width: f32,
    max_width: Option<f32>,
) -> (f32, usize) {
    if text.is_empty() {
        return (0.0, 0);
    }

    let max_chars_per_line = max_width.map(|w| (w / char_width).floor() as usize);

    let mut line_width_max: f32 = 0.0;
    let mut line_count: usize = 0;

    text.lines().for_each(|line| {
        let line_char_count = line.chars().count();

        match max_chars_per_line {
            Some(max_chars) if max_chars > 0 && line_char_count > max_chars => {
                // Word wrap this line
                let wrapped = wrap_line_monospace(line, max_chars);
                wrapped.into_iter().for_each(|wrapped_line| {
                    // Note: Ideally we can get a library to measure all kinds of graphemes.
                    //
                    // I tried this:
                    //
                    // ```rust
                    // let width = unicode_width::UnicodeWidthStr::width_cjk(wrapped_line) as f32 * char_width;
                    // ```
                    //
                    // but it didn't count emoji widths correctly.
                    //
                    // Also tried `string-width`:
                    //
                    // ```rust
                    // let width = string_width::string_width(wrapped_line) as f32 * char_width;
                    // ```

                    let width = line_width_measure(wrapped_line, char_width);
                    line_width_max = line_width_max.max(width);
                    line_count += 1;
                });
            }
            _ => {
                let width = line_width_measure(line, char_width);
                line_width_max = line_width_max.max(width);
                line_count += 1;
            }
        }
    });

    (line_width_max, line_count)
}

/// Returns the width in pixels to display the given line of text.
pub(crate) fn line_width_measure(line: &str, char_width: f32) -> f32 {
    if line.is_empty() {
        return 0.0;
    }

    let mut line_char_column_count = line
        .graphemes(true)
        .map(|grapheme| match emojis::get(grapheme).is_some() {
            true => EMOJI_CHAR_WIDTH,
            false => 1.0f32,
        })
        .sum::<f32>();

    // Add one character width
    //
    // Without this, even with node padding, the text characters reach to both ends
    // of the node, and sometimes the last character wraps down.
    //
    // Note that we shift the x coordinates of each line of text by `0.5 *
    // char_width` in `highlighted_spans_compute`.
    line_char_column_count += 1.0;

    line_char_column_count * char_width
}

/// Wrap text for display, returning owned strings for each line.
pub(crate) fn wrap_text_monospace(text: &str, char_width: f32, max_width: f32) -> Vec<String> {
    let max_chars = (max_width / char_width).floor() as usize;

    if max_chars == 0 {
        return text.lines().map(String::from).collect();
    }

    let mut result = Vec::new();

    text.lines().for_each(|line| {
        let wrapped = wrap_line_monospace(line, max_chars);
        result.extend(wrapped.into_iter().map(String::from));
    });

    if result.is_empty() {
        result.push(String::new());
    }

    result
}

/// Wraps a single line to fit within max_chars characters.
///
/// Tries to break at word boundaries when possible.
pub(crate) fn wrap_line_monospace(line: &str, max_chars: usize) -> Vec<&str> {
    if max_chars == 0 {
        return vec![line];
    }

    let mut result = Vec::new();
    let mut remaining = line;

    while !remaining.is_empty() {
        let char_count = remaining.chars().count();
        if char_count <= max_chars {
            result.push(remaining);
            break;
        }

        // Find a good break point (try to break at whitespace)
        let mut break_at_byte = 0;
        let mut break_at_char = 0;
        let mut last_space_byte = None;
        let mut last_space_char = 0;

        remaining
            .char_indices()
            .enumerate()
            .for_each(|(char_idx, (byte_idx, c))| {
                if char_idx >= max_chars {
                    return;
                }
                if c.is_whitespace() {
                    last_space_byte = Some(byte_idx);
                    last_space_char = char_idx;
                }
                break_at_byte = byte_idx + c.len_utf8();
                break_at_char = char_idx + 1;
            });

        // Prefer breaking at whitespace if we found one in the second half
        let (split_byte, split_char) =
            if let Some(space_byte) = last_space_byte.filter(|_| last_space_char > max_chars / 2) {
                (space_byte, last_space_char)
            } else {
                (break_at_byte, break_at_char)
            };

        if split_char == 0 {
            // Safety: if we can't make progress, just take the whole thing
            result.push(remaining);
            break;
        }

        result.push(&remaining[..split_byte]);
        remaining = remaining[split_byte..].trim_start();
    }

    if result.is_empty() {
        result.push("");
    }

    result
}
