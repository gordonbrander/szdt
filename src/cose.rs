use crate::did::encode_ed25519_did_key;
use crate::ed25519::{PublicKey, SecretKey, get_public_key, sign, vec_to_signature, verify};
use crate::error::{Error, Result};
use serde_cbor::{Value, from_slice, to_vec};
use std::collections::BTreeMap;

// Constants for header parameters
const ALG_HEADER: i128 = 1;
const KID_HEADER: i128 = 4;
const CONTENT_TYPE_HEADER: i128 = 3;

// Algorithm identifier for Ed25519
const ALG_EDDSA: i128 = -8;

pub struct CoseEnvelope {
    protected: BTreeMap<Value, Value>,
    unprotected: BTreeMap<Value, Value>,
    payload: Vec<u8>,
}

impl CoseEnvelope {
    pub fn of(
        content_type: String,
        protected: BTreeMap<Value, Value>,
        unprotected: BTreeMap<Value, Value>,
        payload: Vec<u8>,
    ) -> Self {
        let mut protected = protected;
        protected.insert(
            Value::Integer(CONTENT_TYPE_HEADER),
            Value::Text(content_type),
        );

        CoseEnvelope {
            protected,
            unprotected,
            payload,
        }
    }

    pub fn from_cose_sign1_ed25519(data: &[u8], public_key: &PublicKey) -> Result<Self> {
        // Parse COSE_Sign1 structure
        let cose_sign1: Vec<Value> = from_slice(data)?;

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
        if verify(&to_be_verified, &signature, public_key).is_err() {
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

    pub fn sign_ed25519(mut self, secret_key: &SecretKey) -> Result<Vec<u8>> {
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
            Value::Bytes(signature),
        ];

        // Serialize the COSE_Sign1 structure to CBOR
        Ok(to_vec(&cose_sign1)?)
    }
}
