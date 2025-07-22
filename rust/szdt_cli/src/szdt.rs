use crate::error::Error;
use crate::file::walk_files;
use std::fs::{self, File};
use std::io::BufRead;
use std::path::Path;
use szdt_core::bytes::Bytes;
use szdt_core::cbor_seq::{CborSeqReader, CborSeqWriter};
use szdt_core::contact::Contact;
use szdt_core::content_type;
use szdt_core::ed25519_key_material::Ed25519KeyMaterial;
use szdt_core::memo::Memo;

#[derive(Debug, Clone)]
pub struct ArchiveReceipt {
    pub manifest: Vec<Memo>,
}

/// Write an archive file by reading files from a directory
pub fn archive(
    dir: &Path,
    archive_file: &Path,
    contact: &Contact,
) -> Result<ArchiveReceipt, Error> {
    let key_material = Ed25519KeyMaterial::try_from(contact)?;
    let paths = walk_files(dir)?;

    let archive_file = File::create(archive_file)?;
    let mut archive_writer = CborSeqWriter::new(archive_file);
    let mut manifest: Vec<Memo> = Vec::new();

    for path in &paths {
        // Read file bytes
        let bytes = fs::read(path)?;
        let cbor_bytes = Bytes(bytes);
        let relative_path = path.strip_prefix(dir)?;
        // Create a memo for this file
        let mut memo = Memo::for_body(&cbor_bytes)?;
        // Set file path
        memo.protected.path = Some(relative_path.to_string_lossy().to_string());
        // Set content type (if we can guess it)
        memo.protected.content_type = content_type::guess_from_path(path);
        memo.protected.iss_nickname = Some(contact.nickname.to_string());
        // Sign memo
        memo.sign(&key_material)?;
        // Write memo
        archive_writer.write_block(&memo)?;
        // Write bytes
        archive_writer.write_block(&cbor_bytes)?;
        // Push memo into manifest
        manifest.push(memo);
    }

    archive_writer.flush()?;

    Ok(ArchiveReceipt { manifest })
}

/// Read a pair of memo and bytes from an archive.
/// This function assumes the streaming-friendly sequence layout of:
/// `memo | bytes | memo | bytes | ...`
fn read_archive_memo_pair<R: BufRead>(
    reader: &mut CborSeqReader<R>,
) -> Result<(Memo, Bytes), Error> {
    let memo: Memo = reader.read_block()?;
    let bytes: Bytes = reader.read_block()?;
    Ok((memo, bytes))
}

pub struct Unarchiver<R> {
    reader: CborSeqReader<R>,
}

impl<R: BufRead> Unarchiver<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader: CborSeqReader::new(reader),
        }
    }
}

impl<R: BufRead> Iterator for Unarchiver<R> {
    type Item = Result<(Memo, Bytes), Error>;

    /// Returns an unvalidated pair of `(Memo, Bytes)`
    fn next(&mut self) -> Option<Self::Item> {
        match read_archive_memo_pair(&mut self.reader) {
            Ok((memo, bytes)) => Some(Ok((memo, bytes))),
            Err(Error::Core(szdt_core::error::Error::Eof)) => None,
            Err(err) => Some(Err(err)),
        }
    }
}
