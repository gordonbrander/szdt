use std::io::BufRead;

pub struct Header {
    pub key: String,
    pub value: String,
}

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
    pub fn get_all_values(&self, key: &str) -> Vec<&String> {
        self.0
            .iter()
            .filter(|header| header.key == key)
            .map(|header| &header.value)
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
}
