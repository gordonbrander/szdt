use crate::ed25519_key_material::Ed25519KeyMaterial;
use crate::hash::Hash;
use szdt_core::error::Error as CoreError;
use szdt_core::memo::Memo as CoreMemo;
use wasm_bindgen::prelude::*;

/// WASM wrapper for SZDT memo operations
#[wasm_bindgen]
pub struct Memo {
    inner: CoreMemo,
}

#[wasm_bindgen]
impl Memo {
    /// Create a new memo with the given body hash
    #[wasm_bindgen(constructor)]
    pub fn new(body_hash: Hash) -> Memo {
        Self {
            inner: CoreMemo::new(body_hash.into_core()),
        }
    }

    /// Create a memo for the given body content
    /// Content will be serialized to CBOR and hashed
    #[wasm_bindgen]
    pub fn for_body(content: &[u8]) -> Result<Memo, JsError> {
        let inner = CoreMemo::for_body(content).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Self { inner })
    }

    /// Create a memo for a string body content
    #[wasm_bindgen]
    pub fn for_string(content: &str) -> Result<Memo, JsError> {
        let inner = CoreMemo::for_body(content).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Self { inner })
    }

    /// Create an empty memo (no body content)
    #[wasm_bindgen]
    pub fn empty() -> Memo {
        Self {
            inner: CoreMemo::empty(),
        }
    }

    /// Sign the memo with the given key material
    #[wasm_bindgen]
    pub fn sign(&mut self, key_material: &Ed25519KeyMaterial) -> Result<(), JsError> {
        self.inner
            .sign(key_material.as_core())
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(())
    }

    /// Verify the memo signature
    #[wasm_bindgen]
    pub fn verify(&self) -> Result<bool, JsError> {
        match self.inner.verify() {
            Ok(()) => Ok(true),
            Err(CoreError::MemoUnsigned) => Ok(false),
            Err(CoreError::MemoIssMissing) => Ok(false),
            Err(_) => Ok(false),
        }
    }

    /// Validate the memo (verify signature and check time bounds)
    #[wasm_bindgen]
    pub fn validate(&self, timestamp: Option<u64>) -> Result<bool, JsError> {
        match self.inner.validate(timestamp) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Check if the memo is expired
    #[wasm_bindgen]
    pub fn is_expired(&self, timestamp: Option<u64>) -> bool {
        self.inner.is_expired(timestamp)
    }

    /// Check if the memo is too early (before nbf time)
    #[wasm_bindgen]
    pub fn is_too_early(&self, timestamp: Option<u64>) -> bool {
        self.inner.is_too_early(timestamp)
    }

    /// Serialize the memo to CBOR bytes
    #[wasm_bindgen]
    pub fn to_cbor(&self) -> Result<Vec<u8>, JsError> {
        let bytes =
            serde_ipld_dagcbor::to_vec(&self.inner).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(bytes)
    }

    /// Deserialize a memo from CBOR bytes
    #[wasm_bindgen]
    pub fn from_cbor(data: &[u8]) -> Result<Memo, JsError> {
        let inner: CoreMemo =
            serde_ipld_dagcbor::from_slice(data).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Self { inner })
    }

    /// Check the memo's body hash against a provided hash
    #[wasm_bindgen]
    pub fn checksum(&self, body_hash: &Hash) -> Result<bool, JsError> {
        match self.inner.checksum(body_hash.as_core()) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Get the body hash
    #[wasm_bindgen]
    pub fn body_hash(&self) -> Hash {
        Hash::from_core(self.inner.protected.src)
    }

    /// Get the timestamp when the memo was issued
    #[wasm_bindgen]
    pub fn issued_at(&self) -> u64 {
        self.inner.protected.iat
    }

    /// Get the expiration timestamp (if any)
    #[wasm_bindgen]
    pub fn expires_at(&self) -> Option<u64> {
        self.inner.protected.exp
    }

    /// Get the not-before timestamp (if any)
    #[wasm_bindgen]
    pub fn not_before(&self) -> Option<u64> {
        self.inner.protected.nbf
    }

    /// Get the issuer DID (if signed)
    #[wasm_bindgen]
    pub fn issuer_did(&self) -> Option<String> {
        self.inner.protected.iss.as_ref().map(|did| did.to_string())
    }

    /// Get the content type (if any)
    #[wasm_bindgen]
    pub fn content_type(&self) -> Option<String> {
        self.inner.protected.content_type.clone()
    }

    /// Set the content type
    #[wasm_bindgen]
    pub fn set_content_type(&mut self, content_type: Option<String>) {
        self.inner.protected.content_type = content_type;
    }

    /// Get the file path (if any)
    #[wasm_bindgen]
    pub fn path(&self) -> Option<String> {
        self.inner.protected.path.clone()
    }

    /// Set the file path
    #[wasm_bindgen]
    pub fn set_path(&mut self, path: Option<String>) {
        self.inner.protected.path = path;
    }

    /// Set the expiration time
    #[wasm_bindgen]
    pub fn set_expires_at(&mut self, timestamp: Option<u64>) {
        self.inner.protected.exp = timestamp;
    }

    /// Set the not-before time
    #[wasm_bindgen]
    pub fn set_not_before(&mut self, timestamp: Option<u64>) {
        self.inner.protected.nbf = timestamp;
    }
}

// Internal conversion methods for use within the WASM crate
impl Memo {
    pub fn from_core(core: CoreMemo) -> Self {
        Self { inner: core }
    }

    pub fn into_core(self) -> CoreMemo {
        self.inner
    }

    pub fn as_core(&self) -> &CoreMemo {
        &self.inner
    }
}
