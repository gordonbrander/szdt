use super::hash::Hash;
use super::{error::Error, hashseq::HashSeq};
use std::io::{self, Read, Write};

/// Magic bytes
const MAGIC_BYTES: &[u8] = "SZDT/1.0".as_bytes();
/// Block header size
const HEADER_SIZE: usize = 8;

/// Blake3 Bao Archive writer
pub struct ArchiveWriter<W> {
    writer: W,
}

impl<W: Write> ArchiveWriter<W> {
    /// Construct a new archive writer
    pub fn new(mut writer: W) -> Result<Self, Error> {
        // Write magic bytes
        writer.write_all(MAGIC_BYTES)?;
        Ok(Self { writer })
    }

    /// Writes the bytes as length-prefixed block
    pub fn write_block(&mut self, bytes: &[u8]) -> Result<(), Error> {
        let len: [u8; HEADER_SIZE] = (bytes.len() as u64).to_le_bytes();
        self.writer.write_all(&len)?;
        self.writer.write_all(&bytes)?;
        Ok(())
    }

    /// Unwrap inner writer
    pub fn into_inner(self) -> W {
        self.writer
    }
}

/// Blake3 Bao Archive reader
pub struct ArchiveReader<R: Read> {
    reader: R,
}

impl<R: Read> ArchiveReader<R> {
    pub fn new(mut reader: R) -> Result<Self, Error> {
        // Read and discard magic bytes
        let mut magic_bytes = [0u8; MAGIC_BYTES.len()];
        reader.read_exact(&mut magic_bytes)?;
        if magic_bytes != MAGIC_BYTES {
            return Err(Error::Value(format!(
                "Not an SZDT archive (could not find magic bytes)."
            )));
        }
        Ok(ArchiveReader { reader })
    }

    /// Read next block
    pub fn read_block(&mut self) -> Result<Vec<u8>, Error> {
        let mut len_bytes = [0u8; HEADER_SIZE];
        self.reader.read_exact(&mut len_bytes)?;
        let len = u64::from_le_bytes(len_bytes);
        let mut content: Vec<u8> = vec![0; len as usize];
        self.reader.read_exact(&mut content)?;
        Ok(content)
    }

    /// Get an iterator for the blocks of the archive
    pub fn into_iter(self) -> ArchiveBlockIterator<R> {
        ArchiveBlockIterator(self)
    }

    /// Unwrap inner reader
    pub fn into_inner(self) -> R {
        self.reader
    }
}

impl<R: Read> TryFrom<ArchiveReader<R>> for HashSeq {
    type Error = Error;

    /// Hash blocks of archive, constructing a HashSeq
    fn try_from(archive: ArchiveReader<R>) -> Result<Self, Self::Error> {
        let mut hash_seq = HashSeq::empty();
        for result in archive.into_iter() {
            let block = result?;
            let hash = Hash::new(block);
            hash_seq.append(hash);
        }
        Ok(hash_seq)
    }
}

pub struct ArchiveBlockIterator<R: Read>(ArchiveReader<R>);

impl<R: Read> Iterator for ArchiveBlockIterator<R> {
    type Item = Result<Vec<u8>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.read_block() {
            Ok(vec) => Some(Ok(vec)),
            Err(Error::Io(e)) if e.kind() == io::ErrorKind::UnexpectedEof => None,
            Err(e) => Some(Err(e)),
        }
    }
}
