use ciborium::from_reader_with_buffer;
pub use ciborium::{Value, into_writer};
use std::collections::BTreeMap;
use std::io::{self, Read};

pub struct Headers(BTreeMap<String, Value>);

impl Headers {
    pub fn new(headers: BTreeMap<String, Value>) -> Self {
        Headers(headers)
    }

    /// Parses a CBOR headers map from a byte stream.
    ///
    /// Reader stops reading as soon as the CBOR header map is fully parsed, so
    /// you can use it to parse the headers from a byte stream, and then re-use
    /// the same reader to parse the body of the message.
    ///
    /// Returns the parsed headers as a `BTreeMap<String, Value>`.
    pub fn parse<R: Read>(mut reader: R) -> Result<Headers, io::Error> {
        // Allocate a 32 kb buffer for CBOR parsing
        let mut buffer = vec![0u8; 32 * 1024];

        // Parse the first CBOR value (expected to be a map)
        let cbor = from_reader_with_buffer(&mut reader, &mut buffer)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let Value::Map(pairs) = cbor else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Expected first value to be a CBOR map",
            ));
        };

        let mut headers = BTreeMap::new();
        for (key, value) in pairs {
            if let Value::Text(k) = key {
                headers.insert(k, value);
            }
        }

        Ok(Self(headers))
    }

    /// Serializes the headers as a CBOR map into a byte vector.
    ///
    /// Returns the tagged CBOR map as a `Vec<u8>`.
    pub fn to_header_bytes(&self) -> Result<Vec<u8>, io::Error> {
        let mut output = Vec::new();

        // Convert Headers to a Value::Map for serialization
        let mut map_entries = Vec::new();
        for (key, value) in &self.0 {
            map_entries.push((Value::Text(key.clone()), value.clone()));
        }

        let map = Value::Map(map_entries);

        let tagged = Value::Tag(42, Box::new(map));

        // Serialize the map to the output buffer
        into_writer(&tagged, &mut output)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        Ok(output)
    }
}
