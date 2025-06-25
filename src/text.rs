/// Truncate a string from the left side, adding an ellipsis if necessary.
pub fn truncate_string_left(s: &str, max_chars: usize) -> String {
    if s.len() <= max_chars {
        s.to_string()
    } else {
        format!("…{}", &s[s.len() - (max_chars - 1)..])
    }
}
