use data_encoding::BASE32;
use ed25519_dalek::{SecretKey, Signer, SigningKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use serde_cbor;
use std::collections::HashMap;
use std::fs;
use std::io::Result;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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
    pub fn write_to<W>(&self, writer: &mut W) -> Result<()>
    where
        W: std::io::Write,
    {
        serde_cbor::to_writer(writer, self).map_err(|e| std::io::Error::other(e))
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

/// Deserialized headers. We assign required headers to properties of the struct
/// Additional headers go in to `other`.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Headers {
    /// Content type of body
    pub content_type: String,

    /// Time at which message originated (Unix Epoch in seconds)
    pub created_at: u64,

    /// Public key of sender
    pub pubkey: Option<Vec<u8>>,

    /// Additional headers
    #[serde(flatten)]
    pub other: HashMap<String, serde_cbor::Value>,
}

impl Headers {
    pub fn new(content_type: String) -> Headers {
        Headers {
            content_type,
            created_at: now_epoch_secs(),
            pubkey: None,
            other: HashMap::new(),
        }
    }
}

/// Envelope is the outer wrapper, containing headers, body, and signature
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Envelope {
    /// Cryptographic signature
    pub sig: Option<Vec<u8>>,
    /// Key-value headers
    pub headers: Headers,
    /// Payload (bytes)
    pub body: Vec<u8>,
}

impl Envelope {
    pub fn new(headers: Headers, body: Vec<u8>) -> Envelope {
        Envelope {
            headers,
            body,
            sig: None,
        }
    }

    pub fn of_content_type(content_type: String, body: Vec<u8>) -> Envelope {
        Envelope::new(Headers::new(content_type), body)
    }

    /// Read archive from CBOR
    pub fn read_from<R>(reader: R) -> Result<Envelope>
    where
        R: std::io::Read,
    {
        let envelope: Envelope =
            serde_cbor::from_reader(reader).map_err(|e| std::io::Error::other(e))?;
        Ok(envelope)
    }

    /// Sign the archive with your private key
    pub fn sign(mut self, private_key: &SecretKey) -> Result<Envelope> {
        // Generate a keypair
        let keypair = SigningKey::from_bytes(private_key);

        // Assign pubkey to headers
        self.headers.pubkey = Some(keypair.verifying_key().to_bytes().to_vec());

        // Get bytes for signing
        let signing_bytes = self.to_signing_bytes()?;

        // Sign the bytes
        let signature = keypair.sign(&signing_bytes).to_vec();

        Ok(Envelope {
            sig: Some(signature),
            body: self.body,
            headers: self.headers,
        })
    }

    /// Get the bytes to be signed, an ordered CBOR array of headers and body
    pub fn to_signing_bytes(&self) -> Result<Vec<u8>> {
        // First headers, then body
        let signing_data = (&self.headers, &self.body);
        // Serialize to CBOR bytes
        serde_cbor::to_vec(&signing_data).map_err(|e| std::io::Error::other(e))
    }

    pub fn write_to<W>(&self, writer: W) -> Result<()>
    where
        W: std::io::Write,
    {
        serde_cbor::to_writer(writer, self).map_err(|e| std::io::Error::other(e))
    }
}

impl Serialize for Envelope {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (&self.sig, &self.headers, &self.body).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Envelope {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize as a tuple (signature, headers, body)
        let (sig, headers, body) = Deserialize::deserialize(deserializer)?;

        Ok(Envelope { sig, headers, body })
    }
}

pub fn generate_private_key() -> SigningKey {
    let mut csprng = OsRng;
    SigningKey::generate(&mut csprng)
}

pub fn format_key_base32(key: SigningKey) -> String {
    let key_bytes = key.to_bytes().to_vec();
    BASE32.encode(&key_bytes)
}

pub fn now_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Expected now to be greater than epoch")
        .as_secs()
}

/// Write file to path, creating intermediate directories if needed
pub fn write_file_deep(path: &Path, content: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(path, content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_creation() {
        let headers = Headers::new("application/cbor".to_string());
        let body = vec![1, 2, 3, 4];

        let envelope = Envelope::new(headers, body.clone());

        assert_eq!(envelope.sig, None);
        assert_eq!(envelope.body, body);
        assert_eq!(envelope.headers.content_type, "application/cbor");
    }

    #[test]
    fn test_envelope_signing() {
        let headers = Headers::new("application/cbor".to_string());
        let body = vec![1, 2, 3, 4];

        let envelope = Envelope::new(headers, body);
        let private_key = generate_private_key();
        let secret_key = private_key.to_bytes();

        let signed_envelope = envelope.sign(&secret_key).unwrap();

        assert!(signed_envelope.sig.is_some());
    }

    #[test]
    fn test_envelope_to_signing_bytes() {
        let headers = Headers {
            content_type: "application/cbor".to_string(),
            created_at: 1234567890,
            pubkey: None,
            other: HashMap::new(),
        };
        let body = vec![1, 2, 3, 4];

        let envelope = Envelope::new(headers, body);
        let signing_bytes = envelope.to_signing_bytes().unwrap();

        println!("{:?}", signing_bytes);

        assert!(!signing_bytes.is_empty());
    }
}
