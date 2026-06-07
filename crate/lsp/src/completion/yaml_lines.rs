//! Indentation-aware helpers for scanning YAML line-by-line.
//!
//! These power both the cursor resolver and the dynamic-ID collector, which
//! reason about the (often mid-edit, invalid) buffer structurally rather than
//! by parsing it.

/// Number of leading space characters of `line`.
pub fn indent(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

/// Returns `true` if `line` is blank or a comment.
pub fn is_blank_or_comment(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.is_empty() || trimmed.starts_with('#')
}

/// Returns `true` if `line` is a block sequence item (`- item` or a bare `-`).
pub fn is_list_item(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed == "-" || trimmed.starts_with("- ")
}

/// Returns the map key declared on `line` (`key:` or `key: value`), or `None`
/// for blank / comment / list-item / non-key lines.
pub fn line_map_key(line: &str) -> Option<String> {
    let content = line.trim_start();
    if content.is_empty() || content.starts_with('#') || content.starts_with("- ") {
        return None;
    }

    let colon_idx = content.find(':')?;
    let after = &content[colon_idx + 1..];
    // A key separator is a colon at end-of-line or followed by a space.
    if after.is_empty() || after.starts_with(' ') {
        let key = content[..colon_idx].trim();
        if !key.is_empty() {
            return Some(key.to_string());
        }
    }
    None
}
