use szdt_core::mnemonic::Mnemonic as CoreMnemonic;
use wasm_bindgen::prelude::*;

/// WASM wrapper for BIP39 mnemonic operations
#[wasm_bindgen]
pub struct Mnemonic {
    inner: CoreMnemonic,
}

#[wasm_bindgen]
impl Mnemonic {
    /// Create a mnemonic from entropy bytes
    #[wasm_bindgen]
    pub fn from_entropy(entropy: &[u8]) -> Result<Mnemonic, JsError> {
        // Validate entropy length (must be 16, 20, 24, 28, or 32 bytes for BIP39)
        if ![16, 20, 24, 28, 32].contains(&entropy.len()) {
            return Err(JsError::new(&format!(
                "Entropy must be 16, 20, 24, 28, or 32 bytes, got {}",
                entropy.len()
            )));
        }

        let inner = CoreMnemonic::from_entropy(entropy)?;
        Ok(Self { inner })
    }

    /// Parse a BIP39 mnemonic phrase
    #[wasm_bindgen]
    pub fn parse(mnemonic: &str) -> Result<Mnemonic, JsError> {
        if mnemonic.trim().is_empty() {
            return Err(JsError::new("Mnemonic phrase cannot be empty"));
        }

        // Basic validation - check word count
        let word_count = mnemonic.split_whitespace().count();
        if ![12, 15, 18, 21, 24].contains(&word_count) {
            return Err(JsError::new(&format!(
                "Mnemonic must contain 12, 15, 18, 21, or 24 words, got {}",
                word_count
            )));
        }

        let inner = CoreMnemonic::parse(mnemonic)?;
        Ok(Self { inner })
    }

    /// Get the mnemonic as a string
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }

    /// Get the entropy bytes from the mnemonic
    #[wasm_bindgen]
    pub fn to_entropy(&self) -> Vec<u8> {
        self.inner.to_entropy()
    }

    /// Get the word count of the mnemonic
    #[wasm_bindgen]
    pub fn word_count(&self) -> usize {
        self.inner.to_string().split_whitespace().count()
    }

    /// Validate that a mnemonic phrase is valid BIP39
    #[wasm_bindgen]
    pub fn is_valid(mnemonic: &str) -> bool {
        CoreMnemonic::parse(mnemonic).is_ok()
    }

    /// Generate a random 12-word mnemonic (128 bits entropy)
    #[wasm_bindgen]
    pub fn generate_12_word() -> Result<Mnemonic, JsError> {
        let entropy = generate_random_entropy(16)?;
        Self::from_entropy(&entropy)
    }

    /// Generate a random 15-word mnemonic (160 bits entropy)
    #[wasm_bindgen]
    pub fn generate_15_word() -> Result<Mnemonic, JsError> {
        let entropy = generate_random_entropy(20)?;
        Self::from_entropy(&entropy)
    }

    /// Generate a random 18-word mnemonic (192 bits entropy)
    #[wasm_bindgen]
    pub fn generate_18_word() -> Result<Mnemonic, JsError> {
        let entropy = generate_random_entropy(24)?;
        Self::from_entropy(&entropy)
    }

    /// Generate a random 21-word mnemonic (224 bits entropy)
    #[wasm_bindgen]
    pub fn generate_21_word() -> Result<Mnemonic, JsError> {
        let entropy = generate_random_entropy(28)?;
        Self::from_entropy(&entropy)
    }

    /// Generate a random 24-word mnemonic (256 bits entropy)
    #[wasm_bindgen]
    pub fn generate_24_word() -> Result<Mnemonic, JsError> {
        let entropy = generate_random_entropy(32)?;
        Self::from_entropy(&entropy)
    }
}

// Helper function to generate random entropy
fn generate_random_entropy(length: usize) -> Result<Vec<u8>, JsError> {
    let mut entropy = vec![0u8; length];

    // Use Web Crypto API for random bytes in browser
    let crypto = web_sys::window()
        .ok_or_else(|| JsError::new("No window object available"))?
        .crypto()
        .map_err(|_| JsError::new("Web Crypto API not available"))?;

    crypto
        .get_random_values_with_u8_array(&mut entropy[..])
        .map_err(|_| JsError::new("Failed to generate random bytes"))?;
    Ok(entropy)
}

// Internal conversion methods for use within the WASM crate
impl Mnemonic {
    pub fn from_core(core: CoreMnemonic) -> Self {
        Self { inner: core }
    }

    pub fn into_core(self) -> CoreMnemonic {
        self.inner
    }

    pub fn as_core(&self) -> &CoreMnemonic {
        &self.inner
    }
}
