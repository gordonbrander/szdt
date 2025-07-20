use wasm_bindgen::prelude::*;
use szdt_core::hash::Hash as CoreHash;

/// WASM wrapper for Blake3 hash operations
#[wasm_bindgen]
pub struct Hash {
    inner: CoreHash,
}

#[wasm_bindgen]
impl Hash {
    /// Create a new hash from the provided data
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8]) -> Hash {
        Self {
            inner: CoreHash::new(data),
        }
    }

    /// Create a hash from exactly 32 bytes
    #[wasm_bindgen]
    pub fn from_bytes(bytes: &[u8]) -> Result<Hash, JsError> {
        if bytes.len() != 32 {
            return Err(JsError::new(&format!(
                "Hash must be exactly 32 bytes, got {}", 
                bytes.len()
            )));
        }
        
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(bytes);
        
        Ok(Self {
            inner: CoreHash::from_bytes(hash_bytes),
        })
    }

    /// Get the hash as a byte array
    #[wasm_bindgen]
    pub fn as_bytes(&self) -> Vec<u8> {
        self.inner.as_bytes().to_vec()
    }

    /// Get the hash as a base32 string representation  
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }

    /// Check if two hashes are equal
    #[wasm_bindgen]
    pub fn equals(&self, other: &Hash) -> bool {
        self.inner == other.inner
    }

    /// Create hash from a string (UTF-8 bytes)
    #[wasm_bindgen]
    pub fn from_string(input: &str) -> Hash {
        Self {
            inner: CoreHash::new(input.as_bytes()),
        }
    }
}

// Internal conversion methods for use within the WASM crate
impl Hash {
    pub fn from_core(core: CoreHash) -> Self {
        Self { inner: core }
    }
    
    pub fn into_core(self) -> CoreHash {
        self.inner
    }
    
    pub fn as_core(&self) -> &CoreHash {
        &self.inner
    }
}