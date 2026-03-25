//! Utility functions for string manipulation.

/// Unescapes a string that has been escaped for JSON, reversing the escaping
/// of special characters such as backslashes, quotes, newlines, tabs, and curly
/// braces.
///
/// This is useful for converting escaped strings back to their original form
/// after receiving them from JSON messages.
///
/// The following escape sequences are unescaped:
/// - `\\` becomes `\`
/// - `\"` becomes `"`
/// - `\\n` becomes `\n`
/// - `\\r` becomes `\r`
/// - `\\t` becomes `\t`
/// - `\\{` becomes `{`
/// - `\\}` becomes `}`
///
/// # Examples
/// ```rust
/// let escaped = "Hello \\\"world\\\"!\\nThis is a test.\\t\\{Curly braces\\}";
/// let unescaped = unescape(escaped);
/// assert_eq!(unescaped, "Hello \"world\"!\nThis is a test.\t{Curly braces}");
/// ```
/// # Parameters
/// - `input`: The input string to unescape.
///
/// # Returns
/// - A new string with escape sequences converted back to their original characters.
pub fn unescape(input: &str) -> String {
    input
        .replace("\\\\", "\\")
        .replace("\\\"", "\"")
        .replace("\\n", "\n")
        .replace("\\r", "\r")
        .replace("\\t", "\t")
        .replace("\\{", "{")
        .replace("\\}", "}")
}

/// Returns the singular or plural form of a word based on the provided number.
///
/// # Parameters
/// - `num`: The number to determine singular or plural form.
///   Specifically, 1 or -1 will return the singular form, while all other
///   numbers will return the plural form.
/// - `singular`: The singular form of the word to return if `num` indicates
///   singular (1 or -1).
/// - `plural`: The plural form of the word to return if `num` indicates plural.
///
/// # Returns
/// The singular form if `num` is 1 or -1, otherwise the plural form.
pub fn plurality<'a>(num: isize, singular: &'a str, plural: &'a str) -> &'a str {
    match num {
        1 | -1 => singular,
        _ => plural,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests the `unescape` function to ensure that escaped strings are properly
    /// converted back to their original form.
    #[test]
    fn test_unescape() {
        let input = "Hello \\\"world\\\"!\\nThis is a test.\\t\\{Curly braces\\}";
        let expected = "Hello \"world\"!\nThis is a test.\t{Curly braces}";
        assert_eq!(unescape(input), expected);
    }

    /// Tests the `plurality` function to ensure that it returns the correct
    /// singular or plural form based on the provided number.
    #[test]
    fn test_plurality() {
        assert_eq!(plurality(1, "peer", "peers"), "peer");
        assert_eq!(plurality(-1, "peer", "peers"), "peer");
        assert_eq!(plurality(0, "peer", "peers"), "peers");
        assert_eq!(plurality(2, "peer", "peers"), "peers");
        assert_eq!(plurality(-2, "peer", "peers"), "peers");
    }
}
