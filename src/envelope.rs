use crate::did;
use crate::error::{Error, Result};
use data_encoding::BASE32;
pub use ed25519_dalek::{SecretKey, Signature, SigningKey, VerifyingKey};
use ed25519_dalek::{Signer, Verifier};
use rand::rngs::OsRng;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_cbor;
use std::collections::HashMap;
use std::convert::TryInto;
use std::time::{SystemTime, UNIX_EPOCH};

/// Deserialized headers. We assign required headers to properties of the struct
/// Additional headers go in to `other`.
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Headers {
    /// Content type of body
    pub content_type: String,

    /// Time at which message originated (Unix Epoch in seconds)
    pub created_at: u64,

    /// Public key of sender
    pub did: Option<String>,

    /// Additional headers
    #[serde(flatten)]
    pub other: HashMap<String, serde_cbor::Value>,
}

impl Headers {
    pub fn new(content_type: String) -> Headers {
        Headers {
            content_type,
            created_at: now_epoch_secs(),
            did: None,
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
    pub fn sign(mut self, private_key: &SecretKey) -> Result<Envelope> {
        // Generate a keypair
        let keypair = SigningKey::from_bytes(private_key);

        let did_key = did::encode_ed25519_did_key(&keypair.verifying_key().to_bytes());

        // Assign did for pubkey to headers
        self.headers.did = Some(did_key);

        // Get bytes for signing
        let signing_bytes = self.to_signing_bytes()?;

        // Sign the bytes
        let signature = keypair.sign(&signing_bytes).to_vec();

        Ok(Envelope {
            body: self.body,
            headers: self.headers,
            sig: Some(signature),
        })
    }

    /// Verify the envelope with a public key.
    /// Returns a new Envelope with the signature and public key set.
    pub fn verify_with_key(&self, verifying_key: &VerifyingKey) -> Result<()> {
        let Some(sig) = &self.sig else {
            return Err(Error::SignatureError("No signature".to_string()));
        };

        let Ok(sig_bytes) = sig.as_slice().try_into() else {
            return Err(Error::SignatureError("Invalid signature bytes".to_string()));
        };

        let signature = Signature::from_bytes(sig_bytes);

        // Get bytes for verification
        let signing_bytes = self.to_signing_bytes()?;

        // Verify the signature
        match verifying_key.verify(&signing_bytes, &signature) {
            Ok(()) => Ok(()),
            Err(_) => Err(Error::SignatureError("Signature didn't verify".to_string())),
        }
    }

    /// Verify the envelope using the public key from the headers
    pub fn verify(&self) -> Result<()> {
        // Get public key from headers
        let Some(pubkey) = &self.headers.did else {
            return Err(Error::ValidationError("Missing public key".to_string()));
        };

        // Decode the public key from the DID
        let Some(pubkey) = did::decode_ed25519_did_key(pubkey) else {
            return Err(Error::DecodingError("Invalid public key bytes".to_string()));
        };

        let verifying_key = VerifyingKey::from_bytes(&pubkey)?;
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
        (&self.headers, &self.body, &self.sig).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Envelope {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize as a tuple (signature, headers, body)
        let (headers, body, sig) = Deserialize::deserialize(deserializer)?;

        Ok(Envelope { sig, headers, body })
    }
}

/// Generate a new private key
pub fn generate_private_key() -> SecretKey {
    let mut csprng = OsRng;
    SigningKey::generate(&mut csprng).to_bytes()
}

/// Format a key as a base32 string
pub fn encode_key(key: SecretKey) -> String {
    let key_bytes = key.to_vec();
    BASE32.encode(&key_bytes)
}

pub fn decode_key(key: &str) -> Result<SecretKey> {
    let key_bytes = BASE32.decode(key.as_bytes())?;
    let Ok(private_key) = key_bytes.try_into() else {
        return Err(Error::DecodingError(
            "Could not decode bytes into valid key bytes".to_string(),
        ));
    };
    Ok(private_key)
}

/// Get the current epoch time in seconds
pub fn now_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Expected now to be greater than epoch")
        .as_secs()
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
        let signed_envelope = envelope.sign(&private_key).unwrap();

        assert!(signed_envelope.sig.is_some());
    }

    #[test]
    fn test_envelope_to_signing_bytes() {
        let headers = Headers {
            content_type: "application/cbor".to_string(),
            created_at: 1234567890,
            did: None,
            other: HashMap::new(),
        };
        let body = vec![1, 2, 3, 4];

        let envelope = Envelope::new(headers, body);
        let signing_bytes = envelope.to_signing_bytes().unwrap();

        println!("{:?}", signing_bytes);

        assert!(!signing_bytes.is_empty());
    }

    #[test]
    fn test_envelope_verification_with_key() {
        let headers = Headers::new("application/cbor".to_string());
        let body = vec![1, 2, 3, 4];

        let envelope = Envelope::new(headers, body);
        let private_key = generate_private_key();
        let signing_key = SigningKey::from_bytes(&private_key);
        let verifying_key = signing_key.verifying_key();

        // Sign the envelope
        let signed_envelope = envelope.sign(&private_key).unwrap();

        // Verify the signature with the correct public key
        let verification_result = signed_envelope.verify_with_key(&verifying_key);
        assert!(verification_result.is_ok());

        // Try to verify with a different public key
        let different_private_key = generate_private_key();
        let different_signing_key = SigningKey::from_bytes(&different_private_key);
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
        let private_key = generate_private_key();

        // Sign the envelope
        let signed_envelope = envelope.sign(&private_key).unwrap();

        // Verify the signature with the correct public key
        let verification_result = signed_envelope.verify();
        assert!(verification_result.is_ok());
    }

    #[test]
    fn test_encode_decode_key() {
        // Generate a private key
        let private_key = generate_private_key();

        // Encode the key
        let encoded = encode_key(private_key.clone());

        // Check that encoded key is equal to base32
        assert_eq!(encoded, BASE32.encode(&private_key));

        // Verify we can decode it back
        let decoded = decode_key(&encoded).unwrap();
        assert_eq!(decoded, private_key);
    }
}
