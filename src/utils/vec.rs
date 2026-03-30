//! Utility functions for working with vectors of strings.

#![allow(dead_code)]

/// Trims whitespace from each string in the input vector and filters out empty
/// ones, returning a new vector of cleaned strings.
///
/// # Parameters
/// - `vec`: A reference to a vector of strings that may contain leading or
///   trailing whitespace, and may include empty string elements.
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
mod tests_trim {
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

/// Computes the differences between two vectors, returning a tuple containing
/// two further vectors: the first vector contains elements that are in the
/// first input vector but not in the second, and the second vector contains
/// elements that are in the second input vector but not in the first.
///
/// This function is useful for comparing lists to determine which items have
/// been added or removed.
///
/// # Parameters
/// - `one`: A reference to the first vector.
/// - `other`: A reference to the second vector to compare the first against.
///
/// # Returns
/// A tuple containing two vectors:
/// - The first vector contains elements that are in `one` but not in `other`.
/// - The second vector contains elements that are in `other` but not in `one`.
pub fn get_vec_difference<T: Clone + PartialEq>(one: &[T], other: &[T]) -> (Vec<T>, Vec<T>) {
    let only_one = get_elements_not_in_other_vec(one, other);
    let only_other = get_elements_not_in_other_vec(other, one);
    (only_one, only_other)
}

/// Returns a vector containing the elements that are in the first input vector
/// but not in the second input vector.
///
/// This function is a helper for `get_vec_difference`.
///
/// # Parameters
/// - `one`: A reference to the first vector.
/// - `other`: A reference to the second vector.
///
/// # Returns
/// A vector containing the elements that are in `one` but not in `other`.
pub fn get_elements_not_in_other_vec<T: Clone + PartialEq>(one: &[T], other: &[T]) -> Vec<T> {
    one.iter().filter(|k| !other.contains(k)).cloned().collect()
}

/// Appends the differences between two vectors into the provided
/// destination vectors.
///
/// The first destination vector will receive elements that
/// are in the first source vector but not in the second, and the second
/// destination vector will receive elements that are in the second source vector
/// but not in the first.
///
/// # Parameters
/// - `source_one`: A reference to the first source vector.
/// - `source_other`: A reference to the second source vector.
/// - `dest_one`: A mutable reference to the first destination vector where
///   elements that are in `source_one` but not in `source_other` will be appended.
/// - `dest_other`: A mutable reference to the second destination vector where
///   elements that are in `source_other` but not in `source_one` will be appended.
pub fn append_vec_difference<T: Clone + PartialEq>(
    source_one: &[T],
    source_other: &[T],
    dest_one: &mut Vec<T>,
    dest_other: &mut Vec<T>,
) {
    append_difference_into(dest_one, source_one, source_other);
    append_difference_into(dest_other, source_other, source_one);
}

/// Appends the elements that are in the first input vector but not in the second
/// input vector into the provided destination vector.
///
/// This function is a helper for `append_vec_difference`.
///
/// # Parameters
/// - `dest`: A mutable reference to the destination vector where
///   elements that are in `one` but not in `other` will be appended.
/// - `one`: A reference to the first vector.
/// - `other`: A reference to the second vector.
pub fn append_difference_into<T: Clone + PartialEq>(vec: &mut Vec<T>, one: &[T], other: &[T]) {
    let elements = one.iter().filter(|k| !other.contains(k));
    vec.extend(elements.cloned());
}

#[cfg(test)]
mod tests_vec_difference {
    /// Tests for the `get_vec_difference` function, ensuring that it correctly
    /// computes the differences between two vectors and returns the
    /// expected results in the form of two other vectors: one for elements unique to
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
    /// appends the differences between two vectors into the provided
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

    #[test]
    fn test_get_elements_not_in_other_vec() {
        let previously_lost_keys = vec![
            "peer1".to_string(),
            "peer3".to_string(),
            "peer5".to_string(),
        ];

        let currently_lost_keys = vec![
            "peer2".to_string(),
            "peer3".to_string(),
            "peer4".to_string(),
        ];

        let (only_previously_lost, only_currently_lost) =
            super::get_vec_difference(&previously_lost_keys, &currently_lost_keys);

        assert_eq!(
            only_previously_lost,
            vec!["peer1".to_string(), "peer5".to_string()]
        );
        assert_eq!(
            only_currently_lost,
            vec!["peer2".to_string(), "peer4".to_string()]
        );

        let left = vec![
            "peer1".to_string(),
            "peer2".to_string(),
            "peer3".to_string(),
        ];

        let right = vec!["peer3".to_string(), "peer5".to_string()];
    }
}
