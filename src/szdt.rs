use super::error::Error;
use super::manifest::{self, Manifest, read_file_entries};
use crate::cbor_seq::{CborSeqReader, CborSeqWriter};
use crate::ed25519_key_material::Ed25519KeyMaterial;
use crate::file::{walk_files, write_file_deep};
use crate::link::IntoLink;
use crate::memo::{Memo, SignedMemo};
use crate::util::now;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ArchiveReceipt {
    pub memo: SignedMemo,
    pub manifest: Manifest,
}

/// Write an archive file by reading files from a directory
pub fn archive(
    dir: &Path,
    archive_file: &Path,
    key_material: &Ed25519KeyMaterial,
) -> Result<ArchiveReceipt, Error> {
    let paths = walk_files(dir)?;
    let files = read_file_entries(paths.into_iter())?;

    // Construct manifest
    let manifest = Manifest::new(files.clone());

    // Wrap manifest in a memo
    let mut root_memo = Memo::new(key_material.did()?, manifest.into_link()?);
    root_memo.set_ctype(Some(manifest::CONTENT_TYPE.to_string()));
    // Sign the memo
    let signed_root_memo = root_memo.sign(&key_material)?;

    let archive_file = File::create(archive_file)?;
    let mut archive_writer = CborSeqWriter::new(archive_file);
    // Write the root memo
    archive_writer.write_block(&signed_root_memo)?;
    // Write the manifest
    archive_writer.write_block(&manifest)?;
    // Write everything else
    for file in files {
        let bytes = fs::read(&file.path)?;
        archive_writer.write_block(&bytes)?;
    }

    archive_writer.flush()?;

    Ok(ArchiveReceipt {
        memo: signed_root_memo,
        manifest,
    })
}

#[derive(Debug, Clone)]
pub struct UnarchiveReceipt {
    pub memo: SignedMemo,
    pub manifest: Manifest,
}

/// Unpack an archive into a directory
pub fn unarchive(dir: &Path, archive_file_path: &Path) -> Result<UnarchiveReceipt, Error> {
    let archive_file = BufReader::new(File::open(archive_file_path)?);
    let mut archive_reader = CborSeqReader::new(archive_file);

    let now_time = now();
    let root_memo: SignedMemo = archive_reader.read_block()?;
    // Check if the root memo is valid (signature matches, etc)
    let validated_memo = root_memo.clone().validate(Some(now_time))?;

    let manifest: Manifest = archive_reader.read_block()?;

    // Check memo body matches manifest
    if validated_memo.body != manifest.into_link()? {
        return Err(Error::ArchiveIntegrityError(format!(
            "Archive memo body does not match manifest"
        )));
    }

    let files = manifest.files.clone();

    for file_entry in files {
        // There should be one block for each file
        let bytes: Vec<u8> = archive_reader.read_block()?;
        // Check integrity
        let hash = bytes.into_link()?;
        if hash != file_entry.hash {
            return Err(Error::ArchiveIntegrityError(format!(
                "File integrity error. Hash mismatch for file {}.\n\tExpected: {},\n\tGot: {}",
                file_entry.path.to_string_lossy(),
                file_entry.hash,
                hash
            )));
        }

        let path = dir.join(&file_entry.path);
        write_file_deep(&path, &bytes)?;
    }

    Ok(UnarchiveReceipt {
        memo: root_memo,
        manifest,
    })
}
