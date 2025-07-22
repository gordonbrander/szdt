use std::path::Path;

// Guess the content type from a file extension.
pub fn guess_from_ext(ext: &str) -> Option<String> {
    mime_guess2::from_ext(ext)
        .first()
        .map(|mime| mime.to_string())
}

// Guess the content type from a path.
pub fn guess_from_path(path: &Path) -> Option<String> {
    mime_guess2::from_path(path)
        .first()
        .map(|mime| mime.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_guess_from_ext_html() {
        let result = guess_from_ext("html");
        assert_eq!(result, Some("text/html".to_string()));
    }

    #[test]
    fn test_guess_from_ext_json() {
        let result = guess_from_ext("json");
        assert_eq!(result, Some("application/json".to_string()));
    }

    #[test]
    fn test_guess_from_ext_png() {
        let result = guess_from_ext("png");
        assert_eq!(result, Some("image/png".to_string()));
    }

    #[test]
    fn test_guess_from_ext_unknown() {
        let result = guess_from_ext("xyz123");
        assert_eq!(result, None);
    }

    #[test]
    fn test_guess_from_ext_empty() {
        let result = guess_from_ext("");
        assert_eq!(result, None);
    }

    #[test]
    fn test_guess_from_path_html() {
        let path = Path::new("index.html");
        let result = guess_from_path(path);
        assert_eq!(result, Some("text/html".to_string()));
    }

    #[test]
    fn test_guess_from_path_with_directory() {
        let path = Path::new("/var/www/style.css");
        let result = guess_from_path(path);
        assert_eq!(result, Some("text/css".to_string()));
    }

    #[test]
    fn test_guess_from_path_no_extension() {
        let path = Path::new("README");
        let result = guess_from_path(path);
        assert_eq!(result, None);
    }
}
