pub static ELLIPSIS: &str = "…";

/// Truncate a string from the right side, adding provided ellipsis if necessary.
pub fn truncate(s: &str, max_chars: usize, ellipsis: &str) -> String {
    let string_len = s.chars().count();
    let ellipsis_len = ellipsis.chars().count();
    if string_len <= max_chars {
        s.to_string()
    } else if max_chars > ellipsis_len {
        let truncated_string: String = s.chars().take(max_chars - ellipsis_len).collect();
        format!("{}{}", truncated_string, ellipsis)
    } else {
        let truncated_string: String = s.chars().take(max_chars).collect();
        truncated_string
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_no_truncation_needed() {
        let result = truncate("hello", 10, ELLIPSIS);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_truncate_with_truncation() {
        let result = truncate("hello world", 8, ELLIPSIS);
        assert_eq!(result, "hello w…");
    }

    #[test]
    fn test_truncate_exact_length() {
        let result = truncate("hello", 5, ELLIPSIS);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_truncate_empty_string() {
        let result = truncate("", 5, ELLIPSIS);
        assert_eq!(result, "");
    }

    #[test]
    fn test_truncate_very_long_ellipsis() {
        let result = truncate("h", 8, "123456789");
        assert_eq!(result, "h");
    }
}
