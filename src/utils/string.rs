//! Utility functions for string manipulation, including escaping and
//! unescaping of special characters.

/// Escapes special characters in a string for safe inclusion in JSON, such as
/// backslashes, quotes, newlines, tabs, and curly braces.
///
/// This is important for ensuring that strings are properly formatted when
/// included in JSON messages, such as those used in notifications.
///
/// The following characters are escaped:
/// - `\` becomes `\\`
/// - `"` becomes `\"`
/// - `\n` becomes `\\n`
/// - `\r` becomes `\\r`
/// - `\t` becomes `\\t`
/// - `{` becomes `\\{`
/// - `}` becomes `\\}`
///
/// # Examples
/// ```rust
/// let input = "Hello \"world\"!\nThis is a test.\t{Curly braces}";
/// let escaped = escape_json(input);
/// assert_eq!(escaped, "Hello \\\"world\\\"!\\nThis is a test.\\t\\{Curly braces\\}");
/// ```
///
/// # Parameters
/// - `input`: The input string to escape.
///
/// # Returns
/// - A new string with special characters escaped for JSON.
pub fn escape_json(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
        .replace("{", "\\{")
        .replace("}", "\\}")
}

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
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests the `escape_json` function to ensure that special characters are
    /// properly escaped for JSON formatting.
    #[test]
    fn test_escape_json() {
        let input = "Hello \"world\"!\nThis is a test.\t{Curly braces}";
        let expected = "Hello \\\"world\\\"!\\nThis is a test.\\t\\{Curly braces\\}";
        assert_eq!(escape_json(input), expected);
    }

    /// Tests the `unescape` function to ensure that escaped strings are properly
    /// converted back to their original form.
    #[test]
    fn test_unescape() {
        let input = "Hello \\\"world\\\"!\\nThis is a test.\\t\\{Curly braces\\}";
        let expected = "Hello \"world\"!\nThis is a test.\t{Curly braces}";
        assert_eq!(unescape(input), expected);
    }
}
