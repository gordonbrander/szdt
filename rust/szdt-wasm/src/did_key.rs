use szdt_core::did::DidKey as CoreDidKey;
use wasm_bindgen::prelude::*;

/// WASM wrapper for DID key operations
#[wasm_bindgen]
pub struct DidKey {
    inner: CoreDidKey,
}

#[wasm_bindgen]
impl DidKey {
    /// Create a new DidKey from public key bytes
    #[wasm_bindgen]
    pub fn new(public_key: &[u8]) -> Result<DidKey, JsError> {
        if public_key.len() != 32 {
            return Err(JsError::new(&format!(
                "Public key must be exactly 32 bytes, got {}",
                public_key.len()
            )));
        }

        let inner = CoreDidKey::new(public_key)?;
        Ok(Self { inner })
    }

    /// Parse a did:key URL string into a DidKey
    #[wasm_bindgen]
    pub fn parse(did_key_url: &str) -> Result<DidKey, JsError> {
        if did_key_url.trim().is_empty() {
            return Err(JsError::new("DID key URL cannot be empty"));
        }

        let inner = CoreDidKey::parse(did_key_url)?;
        Ok(Self { inner })
    }

    /// Get the public key bytes
    #[wasm_bindgen]
    pub fn public_key(&self) -> Vec<u8> {
        self.inner.public_key().to_vec()
    }

    /// Get the DID key as a string
    // Allow inherent_to_string so we can expose `.toString()` via wasm_bindgen
    #[allow(clippy::inherent_to_string)]
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }

    /// Check if two DID keys are equal
    #[wasm_bindgen]
    pub fn equals(&self, other: &DidKey) -> bool {
        self.inner == other.inner
    }

    /// Validate that this is a properly formatted DID key
    #[wasm_bindgen]
    pub fn is_valid(did_key_url: &str) -> bool {
        CoreDidKey::parse(did_key_url).is_ok()
    }
}

// Internal conversion methods for use within the WASM crate
impl DidKey {
    pub fn from_core(core: CoreDidKey) -> Self {
        Self { inner: core }
    }

    pub fn into_core(self) -> CoreDidKey {
        self.inner
    }

    pub fn as_core(&self) -> &CoreDidKey {
        &self.inner
    }
}
