use super::archive::ArchiveWriter;
use super::claim_headers::ClaimHeaders;
use super::error::Error;
use super::hash::Hash;
use super::hashseq::HashSeq;
use super::manifest::{FileEntry, Manifest};
use crate::claim::{self, Assertion, Claim, WitnessAssertion};
use crate::ed25519::Ed25519KeyMaterial;
use crate::file::walk_files;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Headers {
    pub claims: Vec<Claim>,
    pub files: Vec<FileEntry>,
}

/// Read file entries from a list of paths
pub fn read_file_entries<I>(paths: I) -> Result<Vec<FileEntry>, Error>
where
    I: Iterator<Item = PathBuf>,
{
    let mut entries: Vec<FileEntry> = Vec::new();
    let mut buf = vec![];
    for path in paths {
        buf.truncate(0);
        let mut file = File::open(&path)?;
        let len = file.read_to_end(&mut buf)?;
        let hash = Hash::new(&buf);
        entries.push(FileEntry::new(hash, len as u64, path));
    }
    Ok(entries)
}

pub fn write_archive(
    dir: &Path,
    archive_file: &Path,
    key_material: Ed25519KeyMaterial,
) -> Result<(), Error> {
    let paths = walk_files(dir)?;
    let files = read_file_entries(paths.into_iter())?;

    // Construct manifest
    let manifest = Manifest::new(files.clone(), None, None);
    let manifest_bytes = serde_ipld_dagcbor::to_vec(&manifest)?;
    let manifest_hash = Hash::new(&manifest_bytes);

    let mut hashseq = HashSeq::empty();
    hashseq.append(manifest_hash);
    for file in &files {
        hashseq.append(file.hash);
    }

    // Create hash for archive segments
    let integrity_hash = Hash::from(hashseq);

    // Witness the hash
    let witness_claim = claim::Builder::new(key_material)?
        .add_ast(Assertion::Witness(WitnessAssertion {
            hash: integrity_hash.as_bytes().to_vec(),
        }))
        .sign()?;

    // Place claim in headers
    let header = ClaimHeaders {
        claims: vec![witness_claim],
    };

    let mut archive_file = ArchiveWriter::new(File::create(archive_file)?)?;
    let header_bytes = serde_ipld_dagcbor::to_vec(&header)?;
    archive_file.write_block(&header_bytes)?;
    for file in files {
        let bytes = fs::read(&file.path)?;
        archive_file.write_block(&bytes)?;
    }

    Ok(())
}
