/// Escapes XML special characters in a string.
pub struct StringXmlEscaper;

impl StringXmlEscaper {
    /// Escapes XML special characters in a string, returning the escaped
    /// result.
    ///
    /// The following characters are replaced with their XML entity equivalents:
    ///
    /// | Character | Replacement |
    /// |-----------|-------------|
    /// | `&`       | `&amp;`     |
    /// | `<`       | `&lt;`      |
    /// | `>`       | `&gt;`      |
    /// | `"`       | `&quot;`    |
    /// | `'`       | `&apos;`    |
    ///
    /// This makes the output safe for use as XML text content or attribute
    /// values.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use disposition_input_ir_rt::StringXmlEscaper;
    ///
    /// assert_eq!(StringXmlEscaper::escape("hello"), "hello");
    /// assert_eq!(StringXmlEscaper::escape("a & b"), "a &amp; b");
    /// assert_eq!(StringXmlEscaper::escape("<tag>"), "&lt;tag&gt;");
    /// assert_eq!(
    ///     StringXmlEscaper::escape(r#"say "hi""#),
    ///     "say &quot;hi&quot;"
    /// );
    /// assert_eq!(StringXmlEscaper::escape("it's"), "it&apos;s");
    /// ```
    pub fn escape(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        s.chars().for_each(|c| match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&apos;"),
            _ => result.push(c),
        });
        result
    }
}
