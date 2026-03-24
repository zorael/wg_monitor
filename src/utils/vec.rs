//! Utility functions for working with vectors of strings, including trimming
//! whitespace, computing differences between vectors, and appending differences
//! into destination vectors.
//!
//! These functions are useful for managing lists of strings such as
//! notification URLs, peer lists, and other collections of string data used
//! across the program.

#![allow(dead_code)]

/// Trims whitespace from each string in the input vector and filters out empty
/// strings, returning a new vector of cleaned strings.
///
/// # Parameters
/// - `vec`: A reference to a vector of strings that may contain leading or
///   trailing whitespace, and may include empty strings.
///
/// # Returns
/// A new vector of strings where each string has been trimmed of whitespace and
/// any empty strings have been removed.
pub fn trim_vec_of_strings(vec: &[String]) -> Vec<String> {
    vec.iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    /// Tests for the `trim_vec_of_strings` function, ensuring that it correctly
    /// trims whitespace from each string and filters out empty strings.
    #[test]
    fn test_trim_vec_of_strings() {
        let input = vec![
            "  https://example.com/webhook1  ".to_string(),
            "https://example.com/webhook2".to_string(),
            "   ".to_string(),
            "".to_string(),
            "https://example.com/webhook3   ".to_string(),
        ];

        let expected = vec![
            "https://example.com/webhook1".to_string(),
            "https://example.com/webhook2".to_string(),
            "https://example.com/webhook3".to_string(),
        ];

        assert_eq!(super::trim_vec_of_strings(&input), expected);
    }
}

/// Computes the differences between two vectors of strings, returning a tuple
/// containing two vectors: the first vector contains elements that are in the
/// first input vector but not in the second, and the second vector contains
/// elements that are in the second input vector but not in the first.
///
/// This function is useful for comparing lists of strings, such as notification
/// URLs or peer lists, to determine which items have been added or removed.
///
/// # Parameters
/// - `one`: A reference to the first vector of strings to compare.
/// - `other`: A reference to the second vector of strings to compare against.
///
/// # Returns
/// A tuple containing two vectors of strings:
/// - The first vector contains elements that are in `one` but not in `other`.
/// - The second vector contains elements that are in `other` but not in `one`.
pub fn get_vec_difference(one: &[String], other: &[String]) -> (Vec<String>, Vec<String>) {
    let only_one = get_elements_not_in_other_vec(one, other);
    let only_other = get_elements_not_in_other_vec(other, one);
    (only_one, only_other)
}

/// Returns a vector of strings containing the elements that are in the first
/// input vector but not in the second input vector.
///
/// This function is a helper for `get_vec_difference` and is used to compute
/// the unique elements in one vector compared to another.
///
/// # Parameters
/// - `one`: A reference to the first vector of strings to compare.
/// - `other`: A reference to the second vector of strings to compare against.
///
/// # Returns
/// A vector of strings containing the elements that are in `one` but not in `other`.
pub fn get_elements_not_in_other_vec(one: &[String], other: &[String]) -> Vec<String> {
    one.iter().filter(|k| !other.contains(k)).cloned().collect()
}

/// Appends the differences between two vectors of strings into the provided
/// destination vectors. The first destination vector will receive elements that
/// are in the first source vector but not in the second, and the second
/// destination vector will receive elements that are in the second source vector
/// but not in the first.
///
/// This function is useful for updating lists of strings based on changes between
/// two source vectors, such as when managing notification URLs or peer lists that
/// may have been modified.
///
/// # Parameters
/// - `source_one`: A reference to the first source vector of strings to compare.
/// - `source_other`: A reference to the second source vector of strings to compare against.
/// - `dest_one`: A mutable reference to the first destination vector of strings where
///   elements that are in `source_one` but not in `source_other` will be appended.
/// - `dest_other`: A mutable reference to the second destination vector of strings where
///   elements that are in `source_other` but not in `source_one` will be appended.
pub fn append_vec_difference(
    source_one: &[String],
    source_other: &[String],
    dest_one: &mut Vec<String>,
    dest_other: &mut Vec<String>,
) {
    append_difference_into(dest_one, source_one, source_other);
    append_difference_into(dest_other, source_other, source_one);
}

/// Appends the elements that are in the first input vector but not in the second
/// input vector into the provided destination vector.
///
/// This function is a helper for `append_vec_difference` and is used to append
/// the unique elements in one vector compared to another into a destination vector.
///
/// # Parameters
/// - `dest`: A mutable reference to the destination vector of strings where
///   elements that are in `one` but not in `other` will be appended.
/// - `one`: A reference to the first vector of strings to compare.
/// - `other`: A reference to the second vector of strings to compare against.
pub fn append_difference_into(vec: &mut Vec<String>, one: &[String], other: &[String]) {
    let elements = one.iter().filter(|k| !other.contains(k));
    vec.extend(elements.cloned());
}

#[cfg(test)]
mod tests_vec {
    /// Tests for the `get_vec_difference` function, ensuring that it correctly
    /// computes the differences between two vectors of strings and returns the
    /// expected results in the form of two vectors: one for elements unique to
    /// the first vector and one for elements unique to the second vector.
    #[test]
    fn test_get_vec_difference() {
        let vec1 = vec![
            "peer1".to_string(),
            "peer2".to_string(),
            "peer3".to_string(),
        ];
        let vec2 = vec![
            "peer2".to_string(),
            "peer3".to_string(),
            "peer4".to_string(),
        ];

        let diff = super::get_vec_difference(&vec1, &vec2);
        assert_eq!(diff.0, vec!["peer1".to_string()]);
        assert_eq!(diff.1, vec!["peer4".to_string()]);
    }

    /// Tests for the `append_vec_difference` function, ensuring that it correctly
    /// appends the differences between two vectors of strings into the provided
    /// destination vectors, resulting in the expected contents of the destination
    /// vectors after the function is called.
    #[test]
    fn test_append_vec_difference() {
        let vec1 = vec![
            "peer1".to_string(),
            "peer2".to_string(),
            "peer3".to_string(),
        ];
        let vec2 = vec![
            "peer2".to_string(),
            "peer3".to_string(),
            "peer4".to_string(),
        ];

        let mut dest_one = Vec::new();
        let mut dest_other = Vec::new();

        super::append_vec_difference(&vec1, &vec2, &mut dest_one, &mut dest_other);
        assert_eq!(dest_one, vec!["peer1".to_string()]);
        assert_eq!(dest_other, vec!["peer4".to_string()]);
    }
}
