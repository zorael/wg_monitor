//! FIXME

/// Trims whitespace from each string in the vector and removes any empty strings, returning a new vector.
pub fn trim_vec_of_strings(vec: &[String]) -> Vec<String> {
    vec.iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    /// Tests for the `trim_vec_of_strings` function.
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

/// Gets the difference between two vectors of strings, returning a tuple of two vectors:
/// the first vector contains elements that are in the first input vector but not in the second,
/// and the second vector contains elements that are in the second input vector but not in the first
pub fn get_vec_difference(one: &[String], other: &[String]) -> (Vec<String>, Vec<String>) {
    let only_one = get_elements_not_in_other_vec(one, other);
    let only_other = get_elements_not_in_other_vec(other, one);
    (only_one, only_other)
}

/// Gets the elements that are in the first vector but not in the second vector, returning a new vector.
pub fn get_elements_not_in_other_vec(one: &[String], other: &[String]) -> Vec<String> {
    one.iter().filter(|k| !other.contains(k)).cloned().collect()
}

/// Appends the difference between two vectors of strings into two destination vectors:
/// the first destination vector will receive elements that are in the first
/// source vector but not in the second, and the second destination vector will
/// receive elements that are in the second source vector but not in the first
/// This is used to efficiently compute differences without needing to create
/// intermediate vectors.
pub fn append_vec_difference(
    source_one: &[String],
    source_other: &[String],
    dest_one: &mut Vec<String>,
    dest_other: &mut Vec<String>,
) {
    append_difference_into(dest_one, source_one, source_other);
    append_difference_into(dest_other, source_other, source_one);
}

/// Appends the elements that are in the first vector but not in the second
/// vector into the provided destination vector.
pub fn append_difference_into(vec: &mut Vec<String>, one: &[String], other: &[String]) {
    let elements = one.iter().filter(|k| !other.contains(k));

    vec.extend(elements.cloned());
}

mod tests_vec {
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
