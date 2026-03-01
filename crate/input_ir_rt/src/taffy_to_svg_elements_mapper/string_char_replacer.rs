/// Replaces a character in a string in-place.
pub struct StringCharReplacer;

impl StringCharReplacer {
    /// Replaces all occurrences of `from` byte with `to` byte in the given
    /// string, mutating it in place.
    ///
    /// # Safety
    ///
    /// This is safe because:
    ///
    /// * Both `from` and `to` must be ASCII bytes (single-byte UTF-8)
    /// * Replacing one ASCII byte with another ASCII byte preserves UTF-8
    ///   validity
    ///
    /// # Panics
    ///
    /// Panics in debug mode if either `from` or `to` is not ASCII.
    pub fn replace_inplace(s: &mut str, from: char, to: char) {
        debug_assert!(from.is_ascii(), "`from` byte must be ASCII");
        debug_assert!(to.is_ascii(), "`to` byte must be ASCII");

        // SAFETY: Replacing ASCII with ASCII preserves UTF-8 validity
        // because ASCII bytes are always single-byte UTF-8 sequences
        // and never appear as continuation bytes in multi-byte sequences.
        unsafe {
            s.as_bytes_mut().iter_mut().for_each(|byte| {
                if *byte == from as u8 {
                    *byte = to as u8;
                }
            });
        }
    }
}
