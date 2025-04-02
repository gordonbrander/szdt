use crate::did::{decode_ed25519_did_key, encode_ed25519_did_key};
use crate::ed25519::{SecretKey, get_public_key, sign, vec_to_signature, verify};
use crate::error::{Error, Result};
use serde::de::DeserializeOwned;
use serde_cbor::{Value, from_slice, to_vec};
use std::collections::BTreeMap;

// Constants for header parameters
const ALG_HEADER: i128 = 1;
const KID_HEADER: i128 = 4;
const CONTENT_TYPE_HEADER: i128 = 3;

// Algorithm identifier for Ed25519
const ALG_EDDSA: i128 = -8;

/// Data structure that may be serialized to/from a COSE_Sign1 CBOR structure.
///
/// CoseEnvelope does not carry the signature data, but may be signed to create
/// valid cryptographically signed COSE_Sign1 CBOR bytes.
pub struct CoseEnvelope {
    pub protected_headers: BTreeMap<Value, Value>,
    pub unprotected_headers: BTreeMap<Value, Value>,
    pub body: Vec<u8>,
}

impl CoseEnvelope {
    pub fn new(
        protected_headers: BTreeMap<Value, Value>,
        unprotected_headers: BTreeMap<Value, Value>,
        body: Vec<u8>,
    ) -> Self {
        CoseEnvelope {
            protected_headers,
            unprotected_headers,
            body,
        }
    }

    /// Create a new envelope of content type.
    /// Creates header maps for both protected and unprotected headers.
    pub fn of_content_type(content_type: String, body: Vec<u8>) -> Self {
        let mut protected_headers = BTreeMap::new();
        protected_headers.insert(
            Value::Integer(CONTENT_TYPE_HEADER),
            Value::Text(content_type),
        );

        CoseEnvelope {
            protected_headers,
            unprotected_headers: BTreeMap::new(),
            body,
        }
    }

    /// Create a CoseEnvelope from COSE_Sign1 CBOR bytes.
    /// Verifies the signature in the COSE data. If successful, returns a CoseEnvelope.
    pub fn from_cose_sign1_ed25519(bytes: &[u8]) -> Result<Self> {
        // Parse COSE_Sign1 structure
        let cose_sign1: Vec<Value> = from_slice(bytes)?;

        if cose_sign1.len() != 4 {
            return Err(Error::ValueError(
                "Invalid COSE_Sign1 structure".to_string(),
            ));
        }

        // Extract components

        // Get protected header bytes
        let protected_headers_bytes = match &cose_sign1[0] {
            Value::Bytes(bytes) => bytes,
            _ => return Err(Error::ValueError("Invalid protected header format".into())),
        };

        // Deserialize protected headers
        let protected_headers: BTreeMap<Value, Value> = from_slice(protected_headers_bytes)?;

        // Get kid header
        let kid = match protected_headers.get(&Value::Integer(KID_HEADER)) {
            Some(Value::Text(kid)) => kid,
            _ => return Err(Error::ValueError("No kid header".into())),
        };

        // Decode Ed25519 public key from DID key in kid header
        let public_key = decode_ed25519_did_key(kid)?;

        let unprotected_headers = match &cose_sign1[1] {
            Value::Map(map) => map.clone(),
            _ => {
                return Err(Error::ValueError(
                    "Invalid unprotected header format".into(),
                ));
            }
        };

        let body = match &cose_sign1[2] {
            Value::Bytes(bytes) => bytes.clone(),
            _ => return Err(Error::ValueError("Invalid payload format".into())),
        };

        let signature_vec: &Vec<u8> = match &cose_sign1[3] {
            Value::Bytes(bytes) => bytes,
            _ => return Err(Error::ValueError("Invalid signature format".into())),
        };

        let signature = vec_to_signature(signature_vec)?;

        // Verify that algorithm is Ed25519
        if let Some(Value::Integer(alg)) = protected_headers.get(&Value::Integer(ALG_HEADER)) {
            if *alg != ALG_EDDSA {
                return Err(Error::ValueError(
                    "Unsupported signing algorithm. Only Ed25519 is supported.".into(),
                ));
            }
        } else {
            return Err(Error::ValueError(
                "Missing algorithm in protected header".into(),
            ));
        }

        // Construct Sig_structure for verification
        let sig_structure = vec![
            Value::Text("Signature1".to_string()),
            Value::Bytes(protected_headers_bytes.clone()),
            Value::Bytes(vec![]), // Empty external_aad
            Value::Bytes(body.clone()),
        ];

        // Serialize Sig_structure to CBOR bytes
        let to_be_verified = to_vec(&sig_structure)?;

        // Verify signature
        if verify(&to_be_verified, &signature, &public_key).is_err() {
            return Err(Error::SignatureVerificationError(
                "Signature verification failed".into(),
            ));
        }

        // Return the envelope
        Ok(CoseEnvelope {
            protected_headers,
            unprotected_headers,
            body,
        })
    }

    /// Sign envelope with private key
    /// Returns valid CBOR COSE_Sign1 bytes, signed with Ed25519 signature scheme.
    pub fn sign_ed25519(mut self, secret_key: &SecretKey) -> Result<Vec<u8>> {
        let public_key = get_public_key(secret_key);

        // Insert DID for public key
        self.protected_headers.insert(
            Value::Integer(KID_HEADER),
            Value::Text(encode_ed25519_did_key(&public_key)),
        );

        // Insert algorithm header hint
        self.protected_headers
            .insert(Value::Integer(ALG_HEADER), Value::Integer(ALG_EDDSA));

        // Serialize protected headers to CBOR bytes (required for COSE_Sign1)
        let protected_bytes = to_vec(&self.protected_headers)?;

        // Prepare the signature input
        // Sig_structure = [
        //   context : "Signature1",
        //   body_protected : bstr,
        //   external_aad : bstr,
        //   payload : bstr
        // ]
        let sig_structure = vec![
            Value::Text("Signature1".to_string()),
            Value::Bytes(protected_bytes.clone()),
            Value::Bytes(vec![]), // Empty external_aad
            Value::Bytes(self.body.clone()),
        ];

        // Serialize Sig_structure to CBOR bytes
        let to_be_signed = to_vec(&sig_structure)?;

        // Sign the data
        let signature = sign(&to_be_signed, secret_key);

        // Create the COSE_Sign1 structure
        let cose_sign1 = vec![
            Value::Bytes(protected_bytes),
            Value::Map(self.unprotected_headers),
            Value::Bytes(self.body),
            Value::Bytes(signature),
        ];

        // Serialize the COSE_Sign1 structure to CBOR
        Ok(to_vec(&cose_sign1)?)
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
