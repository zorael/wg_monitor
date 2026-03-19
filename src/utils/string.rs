//! Some common string manipulation utilities.

/// Escapes common characters in the input string that may interfere with JSON formatting,
/// such as backslashes, quotes, and curly braces.
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

/// Unescapes common escape sequences in a string.
pub fn unescape(input: &str) -> String {
    input
        .replace("\\\\", "\\")
        .replace("\\\"", "\"")
        .replace("\\{", "{")
        .replace("\\}", "}")
        .replace("\\n", "\n")
        .replace("\\r", "\r")
        .replace("\\t", "\t")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests escape_json.
    #[test]
    fn test_escape_json() {
        let input = "Hello \"world\"!\nThis is a test.\t{Curly braces}";
        let expected = "Hello \\\"world\\\"!\\nThis is a test.\\t\\{Curly braces\\}";
        assert_eq!(escape_json(input), expected);
    }

    /// Tests unescape.
    #[test]
    fn test_unescape() {
        let input = "Hello \\\"world\\\"!\\nThis is a test.\\t\\{Curly braces\\}";
        let expected = "Hello \"world\"!\nThis is a test.\t{Curly braces}";
        assert_eq!(unescape(input), expected);
    }
}
