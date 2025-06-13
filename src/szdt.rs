use super::error::Error;
use super::manifest::{self, Manifest};
use crate::cbor_seq::{CborSeqReader, CborSeqWriter};
use crate::ed25519_key_material::Ed25519KeyMaterial;
use crate::file::{walk_files, write_file_deep};
use crate::link::ToLink;
use crate::memo::Memo;
use crate::util::now;
use data_encoding::BASE32_NOPAD_NOCASE;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ArchiveReceipt {
    pub memo: Memo,
    pub manifest: Manifest,
}

/// Write an archive file by reading files from a directory
pub fn archive(
    dir: &Path,
    archive_file: &Path,
    key_material: &Ed25519KeyMaterial,
) -> Result<ArchiveReceipt, Error> {
    let paths = walk_files(dir)?;
    // Construct manifest
    let manifest = Manifest::from_paths(paths.clone().into_iter())?;

    // Wrap manifest in a memo
    let mut root_memo = Memo::for_body(&manifest)?;
    // Set content type
    root_memo.protected.ctype = Some(manifest::CONTENT_TYPE.to_string());
    // Sign the memo
    root_memo.sign(&key_material)?;

    let archive_file = File::create(archive_file)?;
    let mut archive_writer = CborSeqWriter::new(archive_file);
    // Write the root memo
    archive_writer.write_block(&root_memo)?;
    // Write everything else
    for path in &paths {
        let bytes = fs::read(path)?;
        let memo = Memo::for_body(&bytes)?;
        archive_writer.write_block(&memo)?;
    }

    archive_writer.flush()?;

    Ok(ArchiveReceipt {
        memo: root_memo,
        manifest,
    })
}

#[derive(Debug, Clone)]
pub struct UnarchiveReceipt {
    pub memo: Memo,
    pub manifest: Manifest,
}

/// Unpack an archive into a directory
pub fn unarchive(dir: &Path, archive_file_path: &Path) -> Result<UnarchiveReceipt, Error> {
    let archive_file = BufReader::new(File::open(archive_file_path)?);
    let mut archive_reader = CborSeqReader::new(archive_file);

    let now_time = now();
    let root_memo: Memo = archive_reader.read_block()?;
    // Check if the root memo is valid (signature matches, etc)
    root_memo.validate(Some(now_time))?;

    // The next block should be a manifest
    let manifest: Manifest = archive_reader.read_block()?;
    let manifest_hash = manifest.to_link()?;
    // Check memo body matches manifest
    if root_memo.protected.body != manifest_hash {
        return Err(Error::IntegrityError(format!(
            "Manifest does not match memo body hash.\n\tExpected: {},\n\tGot: {}",
            root_memo.protected.body, manifest_hash
        )));
    }

    for file_entry in &manifest.entries {
        // There should two blocks for each file
        let memo: Memo = archive_reader.read_block()?;
        let bytes: Vec<u8> = archive_reader.read_block()?;
        // Check integrity
        let hash = bytes.to_link()?;
        if &hash != &file_entry.hash || &hash != &memo.protected.body {
            let path = file_entry.path.clone().unwrap_or("".to_string());
            return Err(Error::IntegrityError(format!(
                "File does not match memo body hash for path {}.\n\tExpected: {},\n\tGot: {}",
                path, file_entry.hash, hash
            )));
        }

        let hash_base32 = BASE32_NOPAD_NOCASE
            .encode(hash.as_bytes())
            .to_ascii_lowercase();
        let path = dir.join(file_entry.path.as_ref().unwrap_or(&hash_base32));
        write_file_deep(&path, &bytes)?;
    }

    Ok(UnarchiveReceipt {
        memo: root_memo,
        manifest,
    })
}
