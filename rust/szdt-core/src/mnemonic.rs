use crate::error::Error;
use bip39;

pub struct Mnemonic(bip39::Mnemonic);

impl Mnemonic {
    /// Create a new mnemonic from entropy
    pub fn from_entropy(entropy: &[u8]) -> Result<Self, Error> {
        let mnemonic = bip39::Mnemonic::from_entropy(entropy)?;
        Ok(Self(mnemonic))
    }

    /// Parse a BIP39 mnemonic seed phrase into a Mnemonic
    pub fn parse(mnemonic: &str) -> Result<Self, Error> {
        let mnemonic = bip39::Mnemonic::parse_normalized(mnemonic)?;
        Ok(Self(mnemonic))
    }

    pub fn to_entropy(&self) -> Vec<u8> {
        self.0.to_entropy()
    }
}

impl From<Mnemonic> for String {
    fn from(mnemonic: Mnemonic) -> Self {
        mnemonic.to_string()
    }
}

impl TryFrom<&str> for Mnemonic {
    type Error = Error;

    fn try_from(mnemonic: &str) -> Result<Self, Error> {
        Self::parse(mnemonic)
    }
}

impl TryFrom<String> for Mnemonic {
    type Error = Error;

    fn try_from(mnemonic: String) -> Result<Self, Error> {
        Self::parse(&mnemonic)
    }
}

impl std::fmt::Display for Mnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Debug for Mnemonic {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Mnemonic(...)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mnemonic_round_trip() {
        // Test with various entropy sizes
        let entropy_sizes = [16, 20, 24, 28, 32]; // 128, 160, 192, 224, 256 bits

        for &size in &entropy_sizes {
            let entropy = vec![0x42u8; size]; // Use a consistent pattern

            // Create mnemonic from entropy
            let mnemonic = Mnemonic::from_entropy(&entropy).unwrap();

            // Convert to string and back
            let mnemonic_str = mnemonic.to_string();
            let parsed_mnemonic = Mnemonic::parse(&mnemonic_str).unwrap();

            // Check that entropy round-trips correctly
            let recovered_entropy = parsed_mnemonic.to_entropy();
            assert_eq!(
                entropy, recovered_entropy,
                "Entropy round-trip failed for size {size}"
            );

            // Check that string representation round-trips correctly
            assert_eq!(
                mnemonic_str,
                parsed_mnemonic.to_string(),
                "String round-trip failed for size {size}"
            );
        }
    }

    #[test]
    fn test_mnemonic_try_from_conversions() {
        let entropy = vec![0x11u8; 16];
        let original = Mnemonic::from_entropy(&entropy).unwrap();
        let mnemonic_str = original.to_string();

        // Test TryFrom<&str>
        let from_str_ref = Mnemonic::try_from(mnemonic_str.as_str()).unwrap();
        assert_eq!(original.to_entropy(), from_str_ref.to_entropy());

        // Test TryFrom<String>
        let from_string = Mnemonic::try_from(mnemonic_str.clone()).unwrap();
        assert_eq!(original.to_entropy(), from_string.to_entropy());

        // Test From<Mnemonic> for String
        let string_from_mnemonic: String = original.into();
        assert_eq!(mnemonic_str, string_from_mnemonic);
    }
}
