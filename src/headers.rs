use std::io::BufRead;

/// An HTTP-style header.
pub struct Header {
    pub key: String,
    pub value: String,
}

impl Header {
    /// Create a new Header
    pub fn new(key: &str, value: &str) -> Self {
        Header {
            key: key.to_string(),
            value: value.to_string(),
        }
    }

    /// Split the header value by commas
    pub fn split_values(&self) -> Vec<String> {
        self.value.split(',').map(|s| s.to_string()).collect()
    }
}

/// A collection of HTTP-style headers.
pub struct Headers(Vec<Header>);

impl Headers {
    /// Create a new empty Headers
    pub fn new() -> Self {
        Headers(Vec::new())
    }

    /// Streaming parse headers from bytes
    pub fn parse<T>(bytes: &mut T) -> Self
    where
        T: BufRead,
    {
        let mut headers = Vec::new();

        // Split the input string by newlines
        for line in bytes.lines() {
            // Break if line can't be parsed
            let Ok(line) = line else {
                break;
            };

            // Skip empty lines
            if line.is_empty() {
                break;
            }

            // Find the colon separator
            if let Some(index) = line.find(':') {
                let (key, value) = line.split_at(index);

                // Trim the key and skip the colon and any leading whitespace in the value
                let key = key.trim();
                let value = value[1..].trim();

                headers.push(Header {
                    key: key.to_string(),
                    value: value.to_string(),
                });
            }
        }

        Headers(headers)
    }

    /// Get the first value of a header with the given key
    pub fn get_first_value(&self, key: &str) -> Option<&Header> {
        self.0.iter().find(|header| header.key == key)
    }

    /// Get all values for a header with the given key
    /// In addition to collecting all headers with the given key, it also splits
    /// the header value by comma, per RFC 7230 section 3.2.2.
    /// <https://datatracker.ietf.org/doc/html/rfc7230>
    pub fn get_all_values(&self, key: &str) -> Vec<String> {
        self.0
            .iter()
            .filter(|header| header.key == key)
            .flat_map(|header| header.split_values())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Read};

    #[test]
    fn test_parse_http_headers() {
        let mut headers_reader = Cursor::new(
            "Host: example.com\r\n\
            User-Agent: Mozilla/5.0\r\n\
            Accept: text/html\r\n\
            Connection: keep-alive\r\n\
            \r\n\
            Foo"
            .as_bytes(),
        );

        let headers = Headers::parse(&mut headers_reader);

        assert_eq!(headers.0.len(), 4);
        assert_eq!(headers.0[0].key, "Host");
        assert_eq!(headers.0[0].value, "example.com");
        assert_eq!(headers.0[1].key, "User-Agent");
        assert_eq!(headers.0[1].value, "Mozilla/5.0");

        let mut body = String::new();
        _ = headers_reader.read_to_string(&mut body);
        assert_eq!(body, "Foo");
    }

    #[test]
    fn test_get_first_value() {
        let mut headers_reader = Cursor::new(
            "Host: example.com\r\n\
            User-Agent: Mozilla/5.0\r\n\
            Accept: text/html\r\n\
            Connection: keep-alive\r\n\
            \r\n\
            Foo"
            .as_bytes(),
        );

        let headers = Headers::parse(&mut headers_reader);

        let first_host = headers.get_first_value("Host");
        assert!(first_host.is_some());
        assert_eq!(first_host.unwrap().value, "example.com");

        let first_user_agent = headers.get_first_value("User-Agent");
        assert!(first_user_agent.is_some());
        assert_eq!(first_user_agent.unwrap().value, "Mozilla/5.0");

        let non_existent = headers.get_first_value("Non-Existent");
        assert!(non_existent.is_none());
    }

    #[test]
    fn test_get_all_values() {
        let mut headers_reader = Cursor::new(
            "Accept: text/html,application/json\r\n\
            Accept: application/xml\r\n\
            \r\n\
            Body"
                .as_bytes(),
        );

        let headers = Headers::parse(&mut headers_reader);

        let all_accept_values = headers.get_all_values("Accept");
        assert_eq!(
            all_accept_values,
            vec![
                "text/html".to_string(),
                "application/json".to_string(),
                "application/xml".to_string(),
            ]
        );

        let non_existent = headers.get_all_values("Non-Existent");
        assert!(non_existent.is_empty());

        let mut body = String::new();
        _ = headers_reader.read_to_string(&mut body);
        assert_eq!(body, "Body");
    }
}
