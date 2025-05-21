use ed25519_dalek::{
    self, PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, SecretKey, Signature, Signer, SigningKey, Verifier,
    VerifyingKey,
};
use rand::rngs::OsRng;
use thiserror::Error;

pub type PublicKey = [u8; PUBLIC_KEY_LENGTH];
pub type SignatureBytes = [u8; 64];

/// Wraps ed25519 key material, allowing you to sign and verify content
#[derive(Debug, Clone)]
pub struct Ed25519KeyMaterial(PublicKey, Option<SecretKey>);

impl Ed25519KeyMaterial {
    /// Initialize from private key bytes
    pub fn try_from_privkey(privkey: &[u8]) -> Result<Self, Error> {
        let secret_key = to_secret_key(privkey)?;
        let signing_key = SigningKey::from_bytes(&secret_key);
        Ok(Self(
            signing_key.verifying_key().to_bytes(),
            Some(signing_key.to_bytes()),
        ))
    }

    /// Construct key material from a publick key, without a private key
    pub fn try_from_pubkey(pubkey: &[u8]) -> Result<Self, Error> {
        let public_key = to_public_key(pubkey)?;
        Ok(Self(public_key, None))
    }

    /// Get the public key portion
    pub fn pubkey(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    /// Sign payload, returning signature bytes
    pub fn sign(&self, payload: &[u8]) -> Result<Vec<u8>, Error> {
        match &self.1 {
            Some(secret_key) => {
                let signing_key = SigningKey::from_bytes(secret_key);
                Ok(signing_key.sign(payload).to_bytes().to_vec())
            }
            None => Err(Error::SigningError(
                "Can't sign payload. No private key.".to_string(),
            )),
        }
    }

    /// Verify signature
    pub fn verify(&self, payload: &[u8], signature: &[u8]) -> Result<(), Error> {
        let signature = Signature::from_slice(signature)?;
        let verifying_key = VerifyingKey::from_bytes(&self.0)?;
        verifying_key.verify(payload, &signature)?;
        Ok(())
    }
}

impl From<SigningKey> for Ed25519KeyMaterial {
    fn from(signing_key: SigningKey) -> Self {
        Self(
            signing_key.verifying_key().to_bytes(),
            Some(signing_key.to_bytes()),
        )
    }
}

/// Generate a new signing keypair.
/// Returns a tuple of `(pubkey, privkey)`.
pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    (
        signing_key.verifying_key().to_bytes().to_vec(),
        signing_key.to_bytes().to_vec(),
    )
}

/// Convert a Vec<u8> to PublicKey.
/// Returns an error if the input is not exactly 32 bytes.
pub fn to_public_key(bytes: &[u8]) -> Result<PublicKey, Error> {
    if bytes.len() != PUBLIC_KEY_LENGTH {
        return Err(Error::InvalidPublicKey(format!(
            "Public key must be {} bytes, got {}",
            PUBLIC_KEY_LENGTH,
            bytes.len()
        )));
    }

    let mut public_key = [0u8; PUBLIC_KEY_LENGTH];
    public_key.copy_from_slice(bytes);
    Ok(public_key)
}

/// Convert a Vec<u8> to PrivateKey.
/// Returns an error if the input is not exactly 32 bytes.
fn to_secret_key(bytes: &[u8]) -> Result<SecretKey, Error> {
    if bytes.len() != SECRET_KEY_LENGTH {
        return Err(Error::InvalidPrivateKey(format!(
            "Private key must be {} bytes, got {}",
            SECRET_KEY_LENGTH,
            bytes.len()
        )));
    }

    let mut private_key = [0u8; SECRET_KEY_LENGTH];
    private_key.copy_from_slice(bytes);
    Ok(private_key)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Ed25519 error: {0}")]
    Ed25519Error(#[from] ed25519_dalek::ed25519::Error),
    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),
    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),
    #[error("Can't sign payload: {0}")]
    SigningError(String),
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ed25519_key_material_roundtrip() {
        // Generate a signing key
        let (pubkey, privkey) = generate_keypair();

        // Create Ed25519KeyMaterial from the signing key
        let key_material = Ed25519KeyMaterial::try_from_privkey(&privkey).unwrap();

        // Test signing and verification
        let message = b"test message for roundtrip verification";
        let signature = key_material.sign(message).unwrap();

        let key_material_2 = Ed25519KeyMaterial::try_from_pubkey(&pubkey).unwrap();

        // Verify using the same key material
        let result = key_material_2.verify(message, &signature);
        assert!(result.is_ok());

        // Try to verify with a different message
        let wrong_message = b"wrong message".to_vec();
        let result = key_material_2.verify(&wrong_message, &signature);
        assert!(result.is_err());
    }

    #[test]
    fn test_vec_to_public_key() {
        // Valid case
        let valid_bytes = vec![0u8; PUBLIC_KEY_LENGTH];
        let result = to_public_key(&valid_bytes);
        assert!(result.is_ok());

        // Invalid case - wrong length
        let invalid_bytes = vec![0u8; PUBLIC_KEY_LENGTH - 1];
        let result = to_public_key(&invalid_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_vec_to_private_key() {
        // Valid case
        let valid_bytes = vec![0u8; SECRET_KEY_LENGTH];
        let result = to_secret_key(&valid_bytes);
        assert!(result.is_ok());

        // Invalid case - wrong length
        let invalid_bytes = vec![0u8; SECRET_KEY_LENGTH - 1];
        let result = to_secret_key(&invalid_bytes);
        assert!(result.is_err());
    }
}
