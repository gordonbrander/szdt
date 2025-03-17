use crate::error::Result;
use serde::{Deserialize, Serialize};
use serde_cbor;
use std::fs;
use std::path::{Path, PathBuf};

/// MIME type for Safe Zone Data Archives
pub const ARCHIVE_CONTENT_TYPE: &str = "application/vnd.szdat.szdat+cbor";

/// Represents the contents of a file
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

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
pub struct Archive {
    pub nickname: String,
    pub files: Vec<File>,
    pub url: Vec<String>,
}

impl Archive {
    pub fn new(files: Vec<File>) -> Archive {
        Archive {
            files,
            url: Vec::new(),
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
    use crate::envelope::{Envelope, Headers, SigningKey, generate_secret_key};

    #[test]
    fn test_envelope_archive_serialization_deserialization() {
        // Create sample files
        let file1 = File::new(PathBuf::from("test1.txt"), b"This is test file 1".to_vec());
        let file2 = File::new(PathBuf::from("test2.txt"), b"This is test file 2".to_vec());

        // Create an archive with these files
        let original_archive = Archive {
            nickname: "Test Archive".to_string(),
            files: vec![file1, file2],
            url: vec!["https://example.com/archive".to_string()],
        };

        // Serialize the archive to CBOR
        let mut buffer = Vec::new();
        original_archive.write_cbor_to(&mut buffer).unwrap();

        // Create an envelope with the serialized archive
        let envelope = Envelope::of_content_type(ARCHIVE_CONTENT_TYPE, buffer);

        // Deserialize the archive from the envelope
        let deserialized_archive: Archive = envelope.deserialize_body().unwrap();

        // Verify the deserialized archive matches the original
        assert_eq!(deserialized_archive, original_archive);
    }

    #[test]
    fn test_envelope_verification_with_key() {
        let headers = Headers::new("application/cbor".to_string());
        let body = vec![1, 2, 3, 4];

        let envelope = Envelope::new(headers, body);
        let secret_key = generate_secret_key();
        let signing_key = SigningKey::from_bytes(&secret_key);
        let verifying_key = signing_key.verifying_key();

        // Sign the envelope
        let signed_envelope = envelope.sign(&secret_key).unwrap();

        // Verify the signature with the correct public key
        let verification_result = signed_envelope.verify_with_key(&verifying_key);
        assert!(verification_result.is_ok());

        // Try to verify with a different public key
        let different_secret_key = generate_secret_key();
        let different_signing_key = SigningKey::from_bytes(&different_secret_key);
        let different_verifying_key = different_signing_key.verifying_key();
        let wrong_verification = signed_envelope.verify_with_key(&different_verifying_key);
        assert!(wrong_verification.is_err());

        // Test envelope with no signature
        let unsigned_envelope = Envelope::new(
            Headers::new("application/cbor".to_string()),
            vec![5, 6, 7, 8],
        );
        let result = unsigned_envelope.verify_with_key(&verifying_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_envelope_verification_with_header_pubkey() {
        let headers = Headers::new("application/cbor".to_string());
        let body = vec![1, 2, 3, 4];

        let envelope = Envelope::new(headers, body);
        let secret_key = generate_secret_key();

        // Sign the envelope
        let signed_envelope = envelope.sign(&secret_key).unwrap();

        // Verify the signature with the correct public key
        let verification_result = signed_envelope.verify();
        assert!(verification_result.is_ok());
    }
}
