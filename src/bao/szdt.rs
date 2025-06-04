use super::claim_headers::ClaimHeaders;
use super::error::Error;
use super::hash::Hash;
use super::hashseq::HashSeq;
use super::manifest::{Manifest, read_file_entries};
use crate::claim::{self, Assertion, WitnessAssertion};
use crate::ed25519::Ed25519KeyMaterial;
use crate::file::{walk_files, write_file_deep};
use serde::{de::DeserializeOwned, ser::Serialize};
use std::fs::{self, File};
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;

pub struct ArchiveReceipt {
    pub manifest: Manifest,
}

/// Write an archive file by reading files from a directory
pub fn archive(
    dir: &Path,
    archive_file: &Path,
    key_material: Ed25519KeyMaterial,
) -> Result<ArchiveReceipt, Error> {
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

    let archive_file = File::create(archive_file)?;
    let mut archive_writer = SzdtWriter::new(archive_file, &header, &manifest)?;
    for file in files {
        let bytes = fs::read(&file.path)?;
        archive_writer.write_block(&bytes)?;
    }

    Ok(ArchiveReceipt { manifest })
}

pub struct UnarchiveReceipt {}

/// Unpack an archive into a directory
pub fn unarchive(dir: &Path, archive_file_path: &Path) -> Result<UnarchiveReceipt, Error> {
    let archive_file = BufReader::new(File::open(archive_file_path)?);
    let mut archive_reader = SzdtReader::new(archive_file)?;
    let files = archive_reader.manifest.files.clone();

    for file_entry in files {
        // There should be one block for each file
        let bytes: Vec<u8> = match archive_reader.read_block() {
            Ok(bytes) => bytes,
            Err(err) => return Err(err),
        };
        // Check integrity
        let hash = Hash::new(&bytes);
        if hash != file_entry.hash {
            return Err(Error::IntegrityError(format!(
                "File integrity error. Hash mismatch for file {}.\n\tExpected: {},\n\tGot: {}",
                file_entry.path.to_string_lossy(),
                file_entry.hash,
                hash
            )));
        }

        let path = dir.join(&file_entry.path);
        write_file_deep(&path, &bytes)?;
    }

    Ok(UnarchiveReceipt {})
}

/// A specialized reader for deserializes SZDT archives.
/// SZDT archives are CBOR sequences with a particular shape.
pub struct SzdtReader<R> {
    pub headers: ClaimHeaders,
    pub manifest: Manifest,
    reader: R,
}

impl<R: BufRead> SzdtReader<R> {
    pub fn new(mut reader: R) -> Result<Self, Error> {
        let headers: ClaimHeaders = serde_ipld_dagcbor::de::from_reader_once(&mut reader)?;
        let manifest: Manifest = serde_ipld_dagcbor::de::from_reader_once(&mut reader)?;

        Ok(Self {
            headers,
            manifest,
            reader,
        })
    }

    /// Deserialize next block
    pub fn read_block<T: DeserializeOwned>(&mut self) -> Result<T, Error> {
        let result: T = match serde_ipld_dagcbor::de::from_reader_once(&mut self.reader) {
            Ok(value) => value,
            Err(serde_ipld_dagcbor::DecodeError::Eof) => return Err(Error::Eof),
            Err(err) => return Err(Error::CborDecode(err.to_string())),
        };
        Ok(result)
    }

    /// Unwrap inner reader
    pub fn into_inner(self) -> R {
        self.reader
    }
}

/// Represents the metadata portion of an SZDT archive
pub struct SzdtWriter<W> {
    writer: W,
}

impl<W: Write> SzdtWriter<W> {
    pub fn new(mut writer: W, headers: &ClaimHeaders, manifest: &Manifest) -> Result<Self, Error> {
        serde_ipld_dagcbor::ser::to_writer(&mut writer, headers)?;
        serde_ipld_dagcbor::ser::to_writer(&mut writer, manifest)?;

        Ok(Self { writer })
    }

    /// Serialize next block
    pub fn write_block<T: Serialize>(&mut self, block: &T) -> Result<(), Error> {
        serde_ipld_dagcbor::ser::to_writer(&mut self.writer, block)?;
        Ok(())
    }

    /// Unwrap inner writer
    pub fn into_inner(self) -> W {
        self.writer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_szdt_reader_writer_roundtrip() {
        let data = Vec::new();
        let write_cursor = Cursor::new(data);
        let headers = ClaimHeaders::default();
        let manifest = Manifest::new(Vec::new(), None, None);
        let mut writer = SzdtWriter::new(write_cursor, &headers, &manifest).unwrap();
        let bytes = vec![0x01, 0x02, 0x03];
        writer.write_block(&bytes).unwrap();

        let data = writer.into_inner().into_inner();

        let mut read_cursor = Cursor::new(data);
        let mut szdt_reader = SzdtReader::new(&mut read_cursor).unwrap();
        assert_eq!(headers, szdt_reader.headers);
        assert_eq!(manifest, szdt_reader.manifest);
        let bytes = szdt_reader.read_block::<Vec<u8>>().unwrap();
        assert_eq!(bytes, vec![0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_szdt_read_eof() {
        let data = Vec::new();
        let write_cursor = Cursor::new(data);
        let headers = ClaimHeaders::default();
        let manifest = Manifest::new(Vec::new(), None, None);
        let mut writer = SzdtWriter::new(write_cursor, &headers, &manifest).unwrap();
        let bytes = vec![0x01, 0x02, 0x03];
        writer.write_block(&bytes).unwrap();

        let data = writer.into_inner().into_inner();

        let mut read_cursor = Cursor::new(data);
        let mut szdt_reader = SzdtReader::new(&mut read_cursor).unwrap();
        assert_eq!(headers, szdt_reader.headers);
        assert_eq!(manifest, szdt_reader.manifest);
        let bytes = szdt_reader.read_block::<Vec<u8>>().unwrap();
        assert_eq!(bytes, vec![0x01, 0x02, 0x03]);
        let Err(Error::Eof) = szdt_reader.read_block::<Vec<u8>>() else {
            panic!("Should have returned EOF error");
        };
    }
}
