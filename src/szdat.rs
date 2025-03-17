use crate::error::{Error, ErrorKind, Result};
use data_encoding::BASE32;
use ed25519_dalek::{SecretKey, Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_cbor;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs;
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

    /// Create an envelope with the given content type and body
    pub fn of_content_type(content_type: impl Into<String>, body: Vec<u8>) -> Envelope {
        Envelope::new(Headers::new(content_type.into()), body)
    }

    /// Read archive from CBOR bytes reader (e.g. file)
    pub fn read_cbor_from<R>(reader: R) -> Result<Envelope>
    where
        R: std::io::Read,
    {
        let envelope: Envelope =
            serde_cbor::from_reader(reader).map_err(|e| std::io::Error::other(e))?;
        Ok(envelope)
    }

    pub fn write_cbor_to<W>(&self, writer: W) -> Result<()>
    where
        W: std::io::Write,
    {
        serde_cbor::to_writer(writer, self)?;
        Ok(())
    }

    /// Get the bytes to be signed, an ordered CBOR array of headers and body
    pub fn to_signing_bytes(&self) -> Result<Vec<u8>> {
        // First headers, then body
        let signing_data = (&self.headers, &self.body);
        // Serialize to CBOR bytes
        let bytes = serde_cbor::to_vec(&signing_data)?;
        Ok(bytes)
    }

    /// Sign the archive with your private key.
    /// Returns a new Envelope with the signature and public key set.
    pub fn sign(mut self, secret_key: &SecretKey) -> Result<Envelope> {
        // Generate a keypair
        let keypair = SigningKey::from_bytes(secret_key);

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

    /// Verify the envelope with a public key.
    /// Returns a new Envelope with the signature and public key set.
    pub fn verify_with_key(&self, verifying_key: &VerifyingKey) -> Result<()> {
        let Some(sig) = &self.sig else {
            return Err(Error::new("No signature", ErrorKind::SignatureError));
        };

        let Ok(sig_bytes) = sig.as_slice().try_into() else {
            return Err(Error::new(
                "Invalid signature bytes",
                ErrorKind::SignatureError,
            ));
        };

        let signature = Signature::from_bytes(sig_bytes);

        // Get bytes for verification
        let signing_bytes = self.to_signing_bytes()?;

        // Verify the signature
        match verifying_key.verify(&signing_bytes, &signature) {
            Ok(()) => Ok(()),
            Err(_) => Err(Error::new(
                "Signature didn't verify",
                ErrorKind::SignatureError,
            )),
        }
    }

    pub fn verify(&self) -> Result<()> {
        // Get public key from headers
        let Some(pubkey) = &self.headers.pubkey else {
            return Err(Error::new("Missing public key", ErrorKind::ValidationError));
        };

        let Ok(pubkey_slice) = pubkey.as_slice().try_into() else {
            return Err(Error::new(
                "Invalid public key bytes",
                ErrorKind::DecodingError,
            ));
        };

        let verifying_key = VerifyingKey::from_bytes(pubkey_slice)?;
        self.verify_with_key(&verifying_key)
    }

    /// Deserialize the body of the envelope into a given type
    /// The type must implement `DeserializeOwned.
    pub fn deserialize_body<T>(&self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let result = serde_cbor::from_slice(&self.body)?;
        Ok(result)
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

/// Generate a new private key
pub fn generate_secret_key() -> SecretKey {
    let mut csprng = OsRng;
    SigningKey::generate(&mut csprng).to_bytes()
}

/// Format a key as a base32 string
pub fn encode_base32(key: SecretKey) -> String {
    let key_bytes = key.to_vec();
    BASE32.encode(&key_bytes)
}

pub fn decode_base32(key: &str) -> Result<SecretKey> {
    let key_bytes = BASE32.decode(key.as_bytes())?;
    let Ok(secret_key) = key_bytes.try_into() else {
        return Err(Error::new(
            "Could not decode bytes into valid key bytes",
            ErrorKind::DecodingError,
        ));
    };
    Ok(secret_key)
}

/// Get the current epoch time in seconds
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
    fs::write(path, content)?;
    return Ok(());
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
        let secret_key = generate_secret_key();
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

    #[test]
    fn test_archive_serialization_deserialization() {
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
    fn test_envelope_verification() {
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
}
