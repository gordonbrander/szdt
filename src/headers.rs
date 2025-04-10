use std::io::{BufRead, Cursor, Read, Write, copy};

/// An HTTP-style header.
pub struct Header {
    key: String,
    value: String,
}

impl Header {
    /// Create a new Header
    /// Automatically converts the key and value to lowercase
    pub fn new(key: &str, value: &str) -> Self {
        Header {
            key: key.trim().to_lowercase(),
            value: value.trim().to_string(),
        }
    }

    /// Get the header key
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Get the header value.
    /// This method does not parse multiple values out of headers.
    /// To split header value on comma, use `values` method.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Set value of header.
    /// This method sets the literal string value.
    /// To set multiple values, use `set_values` method.
    pub fn set_value(&mut self, value: String) {
        self.value = value;
    }

    /// Split the header value by commas
    pub fn values(&self) -> Vec<String> {
        self.value.split(',').map(|s| s.to_string()).collect()
    }

    /// Set multiple values of header.
    /// This method sets multiple values for the header, joining them with a comma,
    /// per the HTTP specification.
    /// To set the literal string value, use `set_value` method.
    pub fn set_values(&mut self, values: &Vec<String>) {
        self.value = values.join(",");
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

            if let Some((key, value)) = line.split_once(":") {
                headers.push(Header::new(key, value));
            }
        }

        Headers(headers)
    }

    /// Write headers as string to the given writer.
    /// Per HTTP header spec, headers are separated by CLRF and a trailing CLRF
    /// marks the end of headers.
    pub fn write_headers<W>(&self, writer: &mut W) -> std::io::Result<()>
    where
        W: Write,
    {
        for header in &self.0 {
            write!(writer, "{}: {}\r\n", header.key(), header.value())?;
        }
        // Write empty line to indicate end of headers
        write!(writer, "\r\n")?;
        Ok(())
    }

    /// Concat reader contents to the end of headers.
    /// Given a reader (typically source file) and a writer (typically a destination file)
    /// serializes headers, writes them to writer, then copies reader contents to writer in
    /// a streaming fashion.
    pub fn cat<R, W>(&self, reader: &mut R, writer: &mut W) -> std::io::Result<()>
    where
        R: Read,
        W: Write,
    {
        self.write_headers(writer)?;
        copy(reader, writer)?;
        Ok(())
    }

    /// Get the first value of a header with the given key
    /// Note: header keys are normalized to lowercase, so key should be lowercase.
    pub fn first_value(&self, key: &str) -> Option<&Header> {
        self.0.iter().find(|header| header.key() == key)
    }

    /// Get all values for a header with the given key
    /// In addition to collecting all headers with the given key, it also splits
    /// the header value by comma, per RFC 7230 section 3.2.2.
    /// <https://datatracker.ietf.org/doc/html/rfc7230>
    /// Note: header keys are normalized to lowercase, so key should be lowercase.
    pub fn values(&self, key: &str) -> Vec<String> {
        self.0
            .iter()
            .filter(|header| header.key() == key)
            .flat_map(|header| header.values())
            .collect()
    }

    /// Append a header to the headers list.
    pub fn push(&mut self, header: Header) {
        self.0.push(header);
    }

    /// Remove all headers with the given key.
    pub fn drop(&mut self, key: &str) {
        self.0.retain(|header| header.key() != key);
    }
}

impl From<&str> for Headers {
    fn from(header_str: &str) -> Self {
        let mut cursor = Cursor::new(header_str);
        Headers::parse(&mut cursor)
    }
}

impl From<String> for Headers {
    fn from(header_str: String) -> Self {
        let mut cursor = Cursor::new(header_str);
        Headers::parse(&mut cursor)
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
        assert_eq!(headers.0[0].key(), "host");
        assert_eq!(headers.0[0].value(), "example.com");
        assert_eq!(headers.0[1].key(), "user-agent");
        assert_eq!(headers.0[1].value(), "Mozilla/5.0");

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

        let first_host = headers.first_value("host");
        assert!(first_host.is_some());
        assert_eq!(first_host.unwrap().value, "example.com");

        let first_user_agent = headers.first_value("user-agent");
        assert!(first_user_agent.is_some());
        assert_eq!(first_user_agent.unwrap().value, "Mozilla/5.0");

        let non_existent = headers.first_value("non-existent");
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

        let all_accept_values = headers.values("accept");
        assert_eq!(
            all_accept_values,
            vec![
                "text/html".to_string(),
                "application/json".to_string(),
                "application/xml".to_string(),
            ]
        );

        let non_existent = headers.values("non-existent");
        assert!(non_existent.is_empty());

        let mut body = String::new();
        _ = headers_reader.read_to_string(&mut body);
        assert_eq!(body, "Body");
    }

    #[test]
    fn test_write_headers() {
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

        let mut writer: Vec<u8> = Vec::new();
        headers.write_headers(&mut writer).unwrap();

        let expected_headers = "host: example.com\r\n\
            user-agent: Mozilla/5.0\r\n\
            accept: text/html\r\n\
            connection: keep-alive\r\n\
            \r\n";

        assert_eq!(String::from_utf8(writer).unwrap(), expected_headers);
    }

    #[test]
    fn test_cat() {
        let mut content_reader = Cursor::new("Foo".as_bytes());

        let headers = Headers::from(
            "Host: example.com\r\n\
            User-Agent: Mozilla/5.0\r\n\
            Accept: text/html\r\n\
            Connection: keep-alive\r\n",
        );

        let mut writer: Vec<u8> = Vec::new();
        headers.cat(&mut content_reader, &mut writer).unwrap();

        let expected_headers = "host: example.com\r\n\
            user-agent: Mozilla/5.0\r\n\
            accept: text/html\r\n\
            connection: keep-alive\r\n\
            \r\n\
            Foo";

        assert_eq!(String::from_utf8(writer).unwrap(), expected_headers);
    }

    #[test]
    fn test_set_values() {
        let mut header = Header::new("accept", "text/html");

        let values = vec![
            "application/json".to_string(),
            "application/xml".to_string(),
            "text/plain".to_string(),
        ];

        header.set_values(&values);
        assert_eq!(
            header.value(),
            "application/json,application/xml,text/plain"
        );

        // Check that values() method returns the correct values
        let parsed_values = header.values();
        assert_eq!(parsed_values, values);

        // Test with empty vector
        let empty_values: Vec<String> = Vec::new();
        header.set_values(&empty_values);
        assert_eq!(header.value(), "");

        // Test with single value
        let single_value = vec!["text/html".to_string()];
        header.set_values(&single_value);
        assert_eq!(header.value(), "text/html");
    }
}
