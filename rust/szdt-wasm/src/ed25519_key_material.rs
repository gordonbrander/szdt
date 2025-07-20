use wasm_bindgen::prelude::*;
use szdt_core::ed25519_key_material::Ed25519KeyMaterial as CoreKeyMaterial;
use szdt_core::error::Error as CoreError;
use crate::did_key::DidKey;

/// WASM wrapper for Ed25519 key material operations
#[wasm_bindgen]
pub struct Ed25519KeyMaterial {
    inner: CoreKeyMaterial,
}

// Helper function to convert core errors to JsError
fn core_error_to_js_error(err: CoreError) -> JsError {
    JsError::new(&err.to_string())
}

#[wasm_bindgen]
impl Ed25519KeyMaterial {
    /// Generate a new random Ed25519 key pair
    #[wasm_bindgen]
    pub fn generate() -> Result<Ed25519KeyMaterial, JsError> {
        // Generate 32 bytes of entropy using Web Crypto API
        let mut seed = vec![0u8; 32];
        
        let crypto = web_sys::window()
            .ok_or_else(|| JsError::new("No window object available"))?
            .crypto()
            .map_err(|_| JsError::new("Web Crypto API not available"))?;
        
        crypto.get_random_values_with_u8_array(&mut seed[..])
            .map_err(|_| JsError::new("Failed to generate random bytes"))?;
        
        let inner = CoreKeyMaterial::try_from_private_key(&seed).map_err(core_error_to_js_error)?;
        Ok(Self { inner })
    }
    
    /// Create key material from a 32-byte seed
    #[wasm_bindgen]
    pub fn from_seed(seed: &[u8]) -> Result<Ed25519KeyMaterial, JsError> {
        if seed.len() != 32 {
            return Err(JsError::new(&format!(
                "Seed must be exactly 32 bytes, got {}", 
                seed.len()
            )));
        }
        
        let inner = CoreKeyMaterial::try_from_private_key(seed).map_err(core_error_to_js_error)?;
        Ok(Self { inner })
    }
    
    /// Create key material from a BIP39 mnemonic phrase
    #[wasm_bindgen]
    pub fn from_mnemonic(mnemonic: &str) -> Result<Ed25519KeyMaterial, JsError> {
        if mnemonic.trim().is_empty() {
            return Err(JsError::new("Mnemonic cannot be empty"));
        }
        
        let mnemonic_obj = szdt_core::mnemonic::Mnemonic::parse(mnemonic).map_err(core_error_to_js_error)?;
        let inner = CoreKeyMaterial::try_from(&mnemonic_obj).map_err(core_error_to_js_error)?;
        Ok(Self { inner })
    }
    
    /// Create key material from public key only (cannot sign)
    #[wasm_bindgen]
    pub fn from_public_key(public_key: &[u8]) -> Result<Ed25519KeyMaterial, JsError> {
        if public_key.len() != 32 {
            return Err(JsError::new(&format!(
                "Public key must be exactly 32 bytes, got {}", 
                public_key.len()
            )));
        }
        
        let inner = CoreKeyMaterial::try_from_public_key(public_key).map_err(core_error_to_js_error)?;
        Ok(Self { inner })
    }
    
    /// Get the public key as bytes
    #[wasm_bindgen]
    pub fn public_key(&self) -> Vec<u8> {
        self.inner.public_key()
    }
    
    /// Get the private key as bytes (if available)
    #[wasm_bindgen]
    pub fn private_key(&self) -> Option<Vec<u8>> {
        self.inner.private_key()
    }
    
    /// Get the DID key representation
    #[wasm_bindgen]
    pub fn did(&self) -> DidKey {
        DidKey::from_core(self.inner.did())
    }
    
    /// Get the DID key as a string
    #[wasm_bindgen]
    pub fn did_string(&self) -> String {
        self.inner.did().to_string()
    }
    
    /// Sign data with the private key
    #[wasm_bindgen]
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, JsError> {
        if data.is_empty() {
            return Err(JsError::new("Cannot sign empty data"));
        }
        
        let signature = self.inner.sign(data).map_err(core_error_to_js_error)?;
        Ok(signature)
    }
    
    /// Verify a signature against the public key
    #[wasm_bindgen]
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, JsError> {
        if signature.len() != 64 {
            return Err(JsError::new("Signature must be 64 bytes"));
        }
        
        match self.inner.verify(data, signature) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
    
    /// Check if this key material can sign (has private key)
    #[wasm_bindgen]
    pub fn can_sign(&self) -> bool {
        self.inner.private_key().is_some()
    }
}

// Internal conversion methods for use within the WASM crate
impl Ed25519KeyMaterial {
    pub fn from_core(core: CoreKeyMaterial) -> Self {
        Self { inner: core }
    }
    
    pub fn into_core(self) -> CoreKeyMaterial {
        self.inner
    }
    
    pub fn as_core(&self) -> &CoreKeyMaterial {
        &self.inner
    }
}