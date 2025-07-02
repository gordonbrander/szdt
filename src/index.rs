use crate::{cbor_seq::CborSeqReader, error::Error, hash::Hash};
use cbor4ii::core::Value;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{BufRead, Seek},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Index {
    pub entries: HashMap<Hash, IndexEntry>,
}

impl Index {
    pub fn read_from<R: BufRead>(mut reader: CborSeqReader<R>) -> Result<Self, Error> {
        let mut entries: HashMap<Hash, IndexEntry> = HashMap::new();
        let mut start: u64 = 0;
        loop {
            let value: Value = match reader.read_block() {
                Ok(value) => value,
                Err(Error::Eof) => break,
                Err(err) => return Err(err),
            };
            let bytes = serde_ipld_dagcbor::to_vec(&value)?;
            let src = Hash::new(&bytes);
            let length = bytes.len() as u64;
            entries.insert(src, IndexEntry { start, length });

            start += length;
        }
        Ok(Index { entries })
    }

    pub fn get<R: BufRead + Seek>(&self, mut reader: R, src: Hash) -> Result<Vec<u8>, Error> {
        let entry = self
            .entries
            .get(&src)
            .ok_or(Error::NotFound(format!("Entry not found for hash {}", src)))?;

        reader.seek(std::io::SeekFrom::Start(entry.start))?;

        let mut buf = vec![0u8; entry.length as usize];
        reader.read_exact(&mut buf)?;

        let checksum = Hash::new(&buf);
        if src != checksum {
            return Err(Error::IntegrityError(format!(
                "Source didn't match checksum"
            )));
        }

        Ok(buf)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexEntry {
    start: u64,
    length: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cbor_seq::CborSeqWriter;
    use serde::{Deserialize, Serialize};
    use std::io::Cursor;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestData {
        id: u32,
        name: String,
    }

    #[test]
    fn test_index_creation_and_retrieval() {
        let test_data1 = TestData {
            id: 1,
            name: "first".to_string(),
        };
        let test_data2 = TestData {
            id: 2,
            name: "second".to_string(),
        };

        // Write test data to CBOR sequence
        let mut buffer = Vec::new();
        let mut writer = CborSeqWriter::new(&mut buffer);
        writer.write_block(&test_data1).unwrap();
        writer.write_block(&test_data2).unwrap();
        writer.flush().unwrap();

        // Create index from the CBOR sequence
        let cursor = Cursor::new(buffer.clone());
        let reader = CborSeqReader::new(cursor);
        let index = Index::read_from(reader).unwrap();

        // Verify index has correct number of entries
        assert_eq!(index.entries.len(), 2);

        // Get the hash of the first entry
        let first_cbor = serde_ipld_dagcbor::to_vec(&test_data1).unwrap();
        let first_hash = Hash::new(&first_cbor);

        // Retrieve the first entry using the index
        let cursor = Cursor::new(buffer);
        let retrieved_data = index.get(cursor, first_hash).unwrap();

        // Verify the retrieved data matches the original
        assert_eq!(retrieved_data, first_cbor);
    }

    #[test]
    fn test_index_get_not_found() {
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer.clone());
        let reader = CborSeqReader::new(cursor);
        let index = Index::read_from(reader).unwrap();

        // Try to get a non-existent hash
        let fake_hash = Hash::new(b"nonexistent");
        let cursor = Cursor::new(buffer);
        let result = index.get(cursor, fake_hash);

        assert!(matches!(result, Err(Error::NotFound(_))));
    }
}
