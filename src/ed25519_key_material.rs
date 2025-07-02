use crate::did::DidKey;
use crate::ed25519::{
    PrivateKey, PublicKey, derive_public_key, generate_keypair, sign, to_private_key,
    to_public_key, verify,
};
use crate::error::Error;
use crate::mnemonic::Mnemonic;

/// Wraps ed25519 key material, allowing you to
/// - sign and verify
/// - get a DID corresponding to the public key
#[derive(Debug, Clone)]
pub struct Ed25519KeyMaterial {
    public_key: PublicKey,
    private_key: Option<PrivateKey>,
}

impl Ed25519KeyMaterial {
    pub fn generate() -> Self {
        let (public_key, private_key) = generate_keypair();
        Self {
            public_key,
            private_key: Some(private_key),
        }
    }

    /// Initialize from private key bytes
    pub fn try_from_private_key(private_key: &[u8]) -> Result<Self, Error> {
        let private_key = to_private_key(private_key)?;
        let public_key = derive_public_key(&private_key)?;
        Ok(Self {
            public_key: public_key,
            private_key: Some(private_key),
        })
    }

    /// Construct key material from a publick key, without a private key
    pub fn try_from_public_key(pubkey: &[u8]) -> Result<Self, Error> {
        let public_key = to_public_key(pubkey)?;
        Ok(Self {
            public_key,
            private_key: None,
        })
    }

    /// Get the private key portion
    pub fn private_key(&self) -> Option<PrivateKey> {
        self.private_key
    }

    /// Get the public key portion
    pub fn public_key(&self) -> PublicKey {
        self.public_key
    }

    pub fn did(&self) -> DidKey {
        let public_key = self.public_key();
        let did = DidKey::new(&public_key).expect("Should be valid public key");
        did
    }

    /// Sign payload, returning signature bytes
    pub fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, Error> {
        match &self.private_key {
            Some(private_key) => {
                let sig = sign(payload, private_key)?;
                Ok(sig)
            }
            None => Err(Error::PrivateKeyMissing(
                "Can't sign payload. No private key.".to_string(),
            )),
        }
    }

    /// Verify signature
    pub fn verify(&self, payload: &[u8], signature: &[u8]) -> Result<(), Error> {
        verify(payload, signature, &self.public_key)?;
        Ok(())
    }
}

impl TryFrom<&Mnemonic> for Ed25519KeyMaterial {
    type Error = Error;

    fn try_from(value: &Mnemonic) -> Result<Self, Self::Error> {
        let entropy = value.to_entropy();
        Self::try_from_private_key(&entropy)
    }
}

impl TryFrom<&Ed25519KeyMaterial> for Mnemonic {
    type Error = Error;

    fn try_from(key_material: &Ed25519KeyMaterial) -> Result<Self, Self::Error> {
        let Some(private_key) = key_material.private_key() else {
            return Err(Error::PrivateKeyMissing(
                "Can't generate mnemonic. No private key.".to_string(),
            ));
        };
        let mnemonic = Mnemonic::from_entropy(private_key.as_slice())?;
        Ok(mnemonic)
    }
}

impl From<&Ed25519KeyMaterial> for DidKey {
    fn from(key_material: &Ed25519KeyMaterial) -> Self {
        key_material.did()
    }
}

impl TryFrom<&DidKey> for Ed25519KeyMaterial {
    type Error = Error;

    fn try_from(did_key: &DidKey) -> Result<Self, Self::Error> {
        let public_key = did_key.public_key();
        let material = Self::try_from_public_key(public_key)?;
        Ok(material)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ed25519::generate_keypair;

    #[test]
    fn test_ed25519_key_material_roundtrip() {
        // Generate a signing key
        let (pubkey, privkey) = generate_keypair();

        // Create Ed25519KeyMaterial from the signing key
        let key_material = Ed25519KeyMaterial::try_from_private_key(&privkey).unwrap();

        // Test signing and verification
        let message = b"test message for roundtrip verification";
        let signature = key_material.sign(message).unwrap();

        let key_material_2 = Ed25519KeyMaterial::try_from_public_key(&pubkey).unwrap();

        // Verify using the same key material
        let result = key_material_2.verify(message, &signature);
        assert!(result.is_ok());

        // Try to verify with a different message
        let wrong_message = b"wrong message".to_vec();
        let result = key_material_2.verify(&wrong_message, &signature);
        assert!(result.is_err());
    }
}
