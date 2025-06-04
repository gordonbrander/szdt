use super::archive::ArchiveReader;
use super::claim_headers::ClaimHeaders;
use super::error::Error;
use super::manifest::Manifest;
use std::io::Read;

/// Represents the metadata portion of an SZDT archive
pub struct SzdtReader<R: Read> {
    pub headers: ClaimHeaders,
    pub manifest: Manifest,
    reader: ArchiveReader<R>,
}

impl<R: Read> SzdtReader<R> {
    pub fn new(reader: R) -> Result<Self, Error> {
        let mut archive_reader = ArchiveReader::new(reader)?;

        let headers_bytes = archive_reader.read_block()?;
        let headers: ClaimHeaders = serde_ipld_dagcbor::from_slice(&headers_bytes)?;

        let manifest_bytes = archive_reader.read_block()?;
        let manifest: Manifest = serde_ipld_dagcbor::from_slice(&manifest_bytes)?;

        Ok(Self {
            headers,
            manifest,
            reader: archive_reader,
        })
    }

    /// Read next block
    pub fn read_block(&mut self) -> Result<Vec<u8>, Error> {
        self.reader.read_block()
    }

    /// Get an iterator over the blocks in the archive
    pub fn into_iter(self) -> impl Iterator<Item = Result<Vec<u8>, Error>> {
        self.reader.into_iter()
    }

    /// Unwrap inner reader
    pub fn into_inner(self) -> R {
        self.reader.into_inner()
    }
}
