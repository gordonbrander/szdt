use crate::bytes::Bytes;
use crate::cbor_seq::{CborSeqReader, CborSeqWriter};
use crate::ed25519_key_material::Ed25519KeyMaterial;
use crate::error::Error;
use crate::file::{walk_files, write_file_deep};
use crate::link::ToLink;
use crate::manifest::{self, Manifest};
use crate::memo::Memo;
use crate::util::now;
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
    let manifest = Manifest::from_paths(paths.clone().into_iter(), dir)?;

    // Wrap manifest in a memo
    let mut manifest_memo = Memo::for_body(&manifest)?;
    // Set content type
    manifest_memo.protected.content_type = Some(manifest::CONTENT_TYPE.to_string());
    // Sign the memo
    manifest_memo.sign(&key_material)?;

    let archive_file = File::create(archive_file)?;
    let mut archive_writer = CborSeqWriter::new(archive_file);
    // Write the manifest memo
    archive_writer.write_block(&manifest_memo)?;
    // Write the manifest itself
    archive_writer.write_block(&manifest)?;

    // Write file bytes in the order they appear within the resources
    for resource in &manifest.resources {
        // Rebuild the path. We could re-use the paths above, but reading from
        // resource logically ensures that blobs are written in exactly the
        // order they are listed.
        let file_path = &dir.join(&resource.path);
        let bytes = fs::read(file_path)?;
        let cbor_bytes = Bytes(bytes);
        archive_writer.write_block(&cbor_bytes)?;
    }

    archive_writer.flush()?;

    Ok(ArchiveReceipt {
        memo: manifest_memo,
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
    if dir.exists() {
        return Err(Error::Fs(format!(
            "Directory exists: {}",
            dir.to_string_lossy()
        )));
    }

    let archive_file = BufReader::new(File::open(archive_file_path)?);
    let mut archive_reader = CborSeqReader::new(archive_file);

    let now_time = now();
    // First block should be a memo for the manifest
    let manifest_memo: Memo = archive_reader.read_block()?;
    // Check if the root memo is valid (signature matches, etc)
    manifest_memo.validate(Some(now_time))?;

    // The next block should be a manifest
    let manifest: Manifest = archive_reader.read_block()?;
    // Checksum it against memo src
    manifest_memo.checksum(&manifest)?;

    // Now load everything else
    for resource in &manifest.resources {
        let bytes: Bytes = archive_reader.read_block()?;
        let hash = bytes.to_link()?;
        // Check integrity
        if &hash != &resource.src {
            return Err(Error::IntegrityError(format!(
                "Hash does not match for path {}. Expected: {}. Got: {}.",
                &resource.path.to_string_lossy(),
                &resource.src,
                hash
            )));
        }

        let path = dir.join(&resource.path);
        write_file_deep(&path, &bytes.0)?;
    }

    Ok(UnarchiveReceipt {
        memo: manifest_memo,
        manifest,
    })
}
