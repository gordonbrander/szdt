use crate::bytes::Bytes;
use crate::cbor_seq::{CborSeqReader, CborSeqWriter};
use crate::content_type;
use crate::ed25519_key_material::Ed25519KeyMaterial;
use crate::error::Error;
use crate::file::{walk_files, write_file_deep};
use crate::link::ToLink;
use crate::memo::Memo;
use crate::util::now;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ArchiveReceipt {
    pub manifest: Vec<Memo>,
}

/// Write an archive file by reading files from a directory
pub fn archive(
    dir: &Path,
    archive_file: &Path,
    key_material: &Ed25519KeyMaterial,
) -> Result<ArchiveReceipt, Error> {
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
        // Sign memo
        memo.sign(key_material)?;
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

#[derive(Debug, Clone)]
pub struct UnarchiveReceipt {
    pub manifest: Vec<Memo>,
}

/// Read a pair of memo and bytes from an archive.
/// This function assumes the streaming-friendly sequence layout of:
/// ```
/// memo | bytes | memo | bytes | ...
/// ```
fn read_archive_memo_pair<R: BufRead>(
    reader: &mut CborSeqReader<R>,
) -> Result<(Memo, Bytes), Error> {
    let memo: Memo = reader.read_block()?;
    let bytes: Bytes = reader.read_block()?;
    Ok((memo, bytes))
}

/// Unpack an archive into a directory
pub fn unarchive(dir: &Path, archive_file_path: &Path) -> Result<UnarchiveReceipt, Error> {
    if dir.exists() {
        return Err(Error::Fs(format!(
            "Directory exists: {}",
            dir.to_string_lossy()
        )));
    }

    let archive_file = BufReader::new(File::open(archive_file_path)?);
    let mut archive_reader = CborSeqReader::new(archive_file);

    let now_time = now();
    let mut manifest = Vec::new();

    loop {
        match read_archive_memo_pair(&mut archive_reader) {
            Ok((memo, bytes)) => {
                let hash = bytes.to_link()?;
                memo.validate(Some(now_time))?;
                memo.checksum(&hash)?;
                // Use the path in the headers, or else the hash if no path given
                let file_path = memo.protected.path.clone().unwrap_or(hash.to_string());
                let path = dir.join(&file_path);
                let bytes = bytes.into_inner();
                write_file_deep(&path, &bytes)?;
                manifest.push(memo);
            }
            Err(Error::Eof) => {
                break;
            }
            Err(err) => return Err(err),
        };
    }

    Ok(UnarchiveReceipt { manifest })
}
