use super::archive::{ArchiveReader, ArchiveWriter};
use super::error::Error;
use serde::{Serialize, de::DeserializeOwned};
use std::io::{Read, Write};

/// Reader for archive who's first two blocks are CBOR headers.
/// - The first set of headers are unprotected (signature doesn't cover)
/// - The second set of headers are protected (signature covers this block and following blocks)
pub struct HeaderArchive<R: Read, H, P> {
    reader: ArchiveReader<R>,
    unprotected_headers: H,
    protected_headers: P,
}

impl<R: Read, H: DeserializeOwned, P: DeserializeOwned> HeaderArchive<R, H, P> {
    pub fn new(reader: R) -> Result<Self, Error> {
        let mut archive_reader = ArchiveReader::new(reader)?;
        let unprotected_headers_block = archive_reader.read_block()?;
        let unprotected_headers = serde_ipld_dagcbor::from_slice(&unprotected_headers_block)?;
        let protected_headers_block = archive_reader.read_block()?;
        let protected_headers = serde_ipld_dagcbor::from_slice(&protected_headers_block)?;
        Ok(Self {
            reader: archive_reader,
            unprotected_headers,
            protected_headers,
        })
    }
}

impl<R: Read, H, S> HeaderArchive<R, H, S> {
    /// Get reference to deserialized unsigned headers
    pub fn unprotected_headers(&self) -> &H {
        &self.unprotected_headers
    }

    /// Get reference to deserialized signed headers
    pub fn protected_headers(&self) -> &S {
        &self.protected_headers
    }

    /// Read next block from archive
    pub fn read_block(&mut self) -> Result<Vec<u8>, Error> {
        let block = self.reader.read_block()?;
        Ok(block)
    }

    /// Get an iterator for the blocks of this
    pub fn into_iter(self) -> impl Iterator<Item = Result<Vec<u8>, Error>> {
        self.reader.into_iter()
    }

    /// Unwrap inner reader and return it
    pub fn into_inner(self) -> R {
        self.reader.into_inner()
    }
}

/// Writer for an archive who's first block is CBOR metadata
pub struct HeaderArchiveWriter<W: Write> {
    writer: ArchiveWriter<W>,
}

impl<W: Write> HeaderArchiveWriter<W> {
    pub fn new<H: Serialize>(writer: W, unprotected_headers: H) -> Result<Self, Error> {
        let mut archive_writer = ArchiveWriter::new(writer)?;
        // Serialize header to bytess
        let unprotected_header_bytes = serde_ipld_dagcbor::to_vec(&unprotected_headers)?;
        // Write header as first block
        archive_writer.write_block(&unprotected_header_bytes)?;
        return Ok(Self {
            writer: archive_writer,
        });
    }

    /// Write the next block to the archive
    pub fn write_block<R: Read>(&mut self, bytes: &[u8]) -> Result<(), Error> {
        self.writer.write_block(bytes)
    }

    /// Unwrap inner writer
    pub fn into_inner(self) -> W {
        self.writer.into_inner()
    }
}
