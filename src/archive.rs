use crate::error::Result;
use serde::{Deserialize, Serialize};
use serde_cbor;
use std::fs;
use std::path::{Path, PathBuf};

/// MIME type for SZDT Archives
pub const ARCHIVE_CONTENT_TYPE: &str = "application/vnd.szdt.szdt+cbor";

/// Represents the contents of a file
/// The file is inlined as bytes into the archive.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct File {
    /// Suggested path for this file
    pub path: PathBuf,
    /// The raw file bytes
    pub content: Vec<u8>,
}

impl File {
    pub fn new(path: PathBuf, content: Vec<u8>) -> File {
        File { path, content }
    }

    pub fn read(dir: &Path, path: &Path) -> Result<File> {
        let file_path = dir.join(path);
        let content = std::fs::read(file_path)?;
        return Ok(File::new(path.to_path_buf(), content));
    }
}

/// A link to an external resource that can be found in one or more locations.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct Link {
    /// Suggested path for this link
    pub path: PathBuf,
    /// One or more URLs where the file may be found
    pub urls: Vec<String>,
    /// Hash of the file (SHA256 in multihash format)
    pub filehash: Vec<u8>,
}

impl Link {
    pub fn new(path: PathBuf, url: Vec<String>, filehash: Vec<u8>) -> Link {
        Link {
            path,
            urls: url,
            filehash,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
#[serde(tag = "type")]
pub enum FileKind {
    File(File),
    Link(Link),
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct Archive {
    pub nickname: String,
    pub files: Vec<File>,
    pub urls: Vec<String>,
}

impl Archive {
    pub fn new(files: Vec<File>) -> Archive {
        Archive {
            files,
            urls: Vec::new(),
            nickname: String::new(),
        }
    }

    pub fn from_paths(dir: &Path, paths: &[PathBuf]) -> Result<Archive> {
        let mut files = Vec::new();

        for path in paths {
            let file = File::read(dir, path)?;
            files.push(file);
        }

        Ok(Archive::new(files))
    }

    /// Create an archive from the file contents of a directory
    pub fn from_dir(dir: &Path) -> Result<Archive> {
        let mut paths: Vec<PathBuf> = Vec::new();
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_file() {
                paths.push(
                    path.strip_prefix(dir)
                        .map_err(|e| std::io::Error::other(e))?
                        .to_path_buf(),
                );
            }
        }
        return Archive::from_paths(dir, &paths);
    }

    /// Write CBOR data to a writer
    pub fn write_cbor_to<W>(&self, writer: &mut W) -> Result<()>
    where
        W: std::io::Write,
    {
        serde_cbor::to_writer(writer, self)?;
        Ok(())
    }

    /// Write the contents of the archive to individual files in a directory
    pub fn write_archive_contents(&self, dir: &Path) -> Result<()> {
        fs::create_dir_all(dir).expect("Directory should not exist");
        for file in &self.files {
            let mut file_path = dir.to_path_buf();
            file_path.push(&file.path);
            write_file_deep(&file_path, &file.content)?;
        }
        Ok(())
    }
}

/// Write file to path, creating intermediate directories if needed
pub fn write_file_deep(path: &Path, content: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(path, content)?;
    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cose::CoseEnvelope;
    use crate::ed25519::generate_private_key;

    #[test]
    fn test_envelope_archive_serialization_deserialization() {
        // Create sample files
        let file1 = File::new(PathBuf::from("test1.txt"), b"This is test file 1".to_vec());
        let file2 = File::new(PathBuf::from("test2.txt"), b"This is test file 2".to_vec());

        // Create an archive with these files
        let original_archive = Archive {
            nickname: "Test Archive".to_string(),
            files: vec![file1, file2],
            urls: vec!["https://example.com/archive".to_string()],
        };

        // Serialize the archive to CBOR
        let mut buffer = Vec::new();
        original_archive.write_cbor_to(&mut buffer).unwrap();

        // Create an envelope with the serialized archive
        let envelope = CoseEnvelope::of_content_type(ARCHIVE_CONTENT_TYPE.into(), buffer);

        // Deserialize the archive from the envelope
        let deserialized_archive: Archive = envelope.deserialize_body().unwrap();

        // Verify the deserialized archive matches the original
        assert_eq!(deserialized_archive, original_archive);
    }

    #[test]
    fn test_envelope_verification() {
        let body = vec![1, 2, 3, 4];

        let envelope = CoseEnvelope::of_content_type(ARCHIVE_CONTENT_TYPE.into(), body.clone());
        let private_key = generate_private_key();

        // Sign the envelope
        let signed_data = envelope.sign_ed25519(&private_key).unwrap();

        let envelope2 = CoseEnvelope::from_cose_sign1_ed25519(&signed_data).unwrap();

        let body2: Vec<u8> = envelope2.deserialize_body().unwrap();

        assert_eq!(body, body2);
    }
}
