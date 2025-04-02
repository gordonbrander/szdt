use crate::did::{decode_ed25519_did_key, encode_ed25519_did_key};
use crate::ed25519::{SecretKey, get_public_key, sign, vec_to_signature, verify};
use crate::error::{Error, Result};
use serde::de::DeserializeOwned;
use serde_cbor::tags::Tagged;
use serde_cbor::{Value, from_slice, to_vec};
use std::collections::BTreeMap;

// Constants for header parameters
const ALG_HEADER: i128 = 1;
const KID_HEADER: i128 = 4;
const CONTENT_TYPE_HEADER: i128 = 3;

// Algorithm identifier for Ed25519
const ALG_EDDSA: i128 = -8;

const TAG_COSE_SIGN1: u64 = 18;

/// Data structure that may be serialized to/from a COSE_Sign1 CBOR structure.
///
/// CoseEnvelope does not carry the signature data, but may be signed to create
/// valid cryptographically signed COSE_Sign1 CBOR bytes.
pub struct CoseEnvelope {
    pub protected: BTreeMap<Value, Value>,
    pub unprotected: BTreeMap<Value, Value>,
    pub payload: Vec<u8>,
}

impl CoseEnvelope {
    pub fn new(
        protected: BTreeMap<Value, Value>,
        unprotected: BTreeMap<Value, Value>,
        payload: Vec<u8>,
    ) -> Self {
        CoseEnvelope {
            protected,
            unprotected,
            payload,
        }
    }

    /// Create a new envelope of content type.
    /// Creates header maps for both protected and unprotected headers.
    pub fn of_content_type(content_type: String, payload: Vec<u8>) -> Self {
        let mut protected = BTreeMap::new();
        protected.insert(
            Value::Integer(CONTENT_TYPE_HEADER),
            Value::Text(content_type),
        );

        CoseEnvelope {
            protected,
            unprotected: BTreeMap::new(),
            payload,
        }
    }

    /// Create a CoseEnvelope from COSE_Sign1 CBOR bytes.
    /// Verifies the signature in the COSE data. If successful, returns a CoseEnvelope.
    pub fn from_cose_sign1(bytes: &[u8]) -> Result<Self> {
        // Parse COSE_Sign1 structure
        let cose_sign1: Vec<Value> = from_slice(bytes)?;

        if cose_sign1.len() != 4 {
            return Err(Error::ValueError(
                "Invalid COSE_Sign1 structure".to_string(),
            ));
        }

        // Extract components

        // Get protected header bytes
        let protected_bytes = match &cose_sign1[0] {
            Value::Bytes(bytes) => bytes,
            _ => return Err(Error::ValueError("Invalid protected header format".into())),
        };

        // Deserialize protected headers
        let protected: BTreeMap<Value, Value> = from_slice(protected_bytes)?;

        // Get kid header
        let kid = match protected.get(&Value::Integer(KID_HEADER)) {
            Some(Value::Text(kid)) => kid,
            _ => return Err(Error::ValueError("No kid header".into())),
        };

        // Decode Ed25519 public key from DID key in kid header
        let public_key = decode_ed25519_did_key(kid)?;

        let unprotected = match &cose_sign1[1] {
            Value::Map(map) => map.clone(),
            _ => {
                return Err(Error::ValueError(
                    "Invalid unprotected header format".into(),
                ));
            }
        };

        let payload = match &cose_sign1[2] {
            Value::Bytes(bytes) => bytes.clone(),
            _ => return Err(Error::ValueError("Invalid payload format".into())),
        };

        let signature_vec: &Vec<u8> = match &cose_sign1[3] {
            Value::Bytes(bytes) => bytes,
            _ => return Err(Error::ValueError("Invalid signature format".into())),
        };

        let signature = vec_to_signature(signature_vec)?;

        // Verify that algorithm is Ed25519
        if let Some(Value::Integer(alg)) = protected.get(&Value::Integer(ALG_HEADER)) {
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
            Value::Bytes(protected_bytes.clone()),
            Value::Bytes(vec![]), // Empty external_aad
            Value::Bytes(payload.clone()),
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
            protected,
            unprotected,
            payload,
        })
    }

    /// Sign envelope with private key
    /// Returns valid CBOR COSE_Sign1 bytes, signed with Ed25519 signature scheme.
    pub fn to_cose_sign1_ed25519(mut self, secret_key: &SecretKey) -> Result<Vec<u8>> {
        let public_key = get_public_key(secret_key);

        // Insert DID for public key
        self.protected.insert(
            Value::Integer(KID_HEADER),
            Value::Text(encode_ed25519_did_key(&public_key)),
        );

        // Insert algorithm header hint
        self.protected
            .insert(Value::Integer(ALG_HEADER), Value::Integer(ALG_EDDSA));

        // Serialize protected headers to CBOR bytes (required for COSE_Sign1)
        let protected_bytes = to_vec(&self.protected)?;

        // Prepare the signature input
        // See <https://www.rfc-editor.org/rfc/rfc9052.html#section-4.4>
        let sig_structure = vec![
            Value::Text("Signature1".to_string()),
            Value::Bytes(protected_bytes.clone()),
            Value::Bytes(vec![]), // Empty external_aad
            Value::Bytes(self.payload.clone()),
        ];

        // Serialize Sig_structure to CBOR bytes
        let to_be_signed = to_vec(&sig_structure)?;

        // Sign the data
        let signature = sign(&to_be_signed, secret_key);

        // Create the COSE_Sign1 structure
        let cose_sign1 = vec![
            Value::Bytes(protected_bytes),
            Value::Map(self.unprotected),
            Value::Bytes(self.payload),
            Value::Bytes(signature.to_vec()),
        ];

        // Tag as COSE_Sign1
        let cose_sign1_tagged = Tagged::new(Some(TAG_COSE_SIGN1), cose_sign1);

        // Serialize the COSE_Sign1 structure to CBOR
        Ok(to_vec(&cose_sign1_tagged)?)
    }

    /// Deserialize the body of the envelope into a given type
    /// The type must implement `DeserializeOwned.
    pub fn deserialize_payload<T>(&self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let result = serde_cbor::from_slice(&self.payload)?;
        Ok(result)
    }
}
