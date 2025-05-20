use ed25519_dalek::{self, PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, Signature};
pub use ed25519_dalek::{SecretKey, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use thiserror::Error;

pub type PublicKey = [u8; PUBLIC_KEY_LENGTH];
pub type SignatureBytes = [u8; 64];

/// Get publick key from secret key
pub fn get_public_key(secret_key: &SecretKey) -> PublicKey {
    SigningKey::from_bytes(secret_key)
        .verifying_key()
        .to_bytes()
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

/// Sign bytes with a private key.
/// Returns signature bytes.
pub fn sign(bytes: &[u8], secret_key: &SecretKey) -> SignatureBytes {
    // Generate a keypair
    let signing_key = SigningKey::from_bytes(secret_key);
    signing_key.sign(&bytes).to_bytes()
}

/// Generate a new signing key
pub fn generate_signing_key() -> SigningKey {
    let mut csprng = OsRng;
    SigningKey::generate(&mut csprng)
}

/// Convert a Vec<u8> to PrivateKey.
/// Returns an error if the input is not exactly 32 bytes.
pub fn to_secret_key(bytes: &[u8]) -> Result<SecretKey, Error> {
    if bytes.len() != SECRET_KEY_LENGTH {
        return Err(Error::InvalidSecretKey(format!(
            "Private key must be {} bytes, got {}",
            SECRET_KEY_LENGTH,
            bytes.len()
        )));
    }

    let mut private_key = [0u8; SECRET_KEY_LENGTH];
    private_key.copy_from_slice(bytes);
    Ok(private_key)
}

/// Convert a Vec<u8> to SignatureBytes.
/// Returns an error if the input is not exactly 64 bytes.
pub fn to_signature_bytes(bytes: &[u8]) -> Result<SignatureBytes, Error> {
    if bytes.len() != 64 {
        return Err(Error::InvalidSignature(format!(
            "Signature must be 64 bytes, got {}",
            bytes.len()
        )));
    }

    let mut signature = [0u8; 64];
    signature.copy_from_slice(bytes);
    Ok(signature)
}

/// Verify bytes with signature and public key.
pub fn verify(
    bytes: &Vec<u8>,
    signature_bytes: &SignatureBytes,
    public_key: &PublicKey,
) -> Result<(), Error> {
    let verifying_key = VerifyingKey::from_bytes(public_key)?;
    let signature = Signature::from_bytes(signature_bytes);
    // Verify the signature
    verifying_key.verify(bytes, &signature)?;
    Ok(())
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Ed25519 error: {0}")]
    Ed25519Error(#[from] ed25519_dalek::ed25519::Error),
    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),
    #[error("Invalid secret key: {0}")]
    InvalidSecretKey(String),
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation_and_signing() {
        // Generate a private key
        let signing_key = generate_signing_key();
        let secret_key = signing_key.to_bytes();

        // Get the corresponding public key
        let public_key = signing_key.verifying_key().to_bytes();

        // Create a message and sign it
        let message = b"test message".to_vec();
        let signature = sign(&message, &secret_key);

        // Verify the signature
        let result = verify(&message, &signature, &public_key);
        assert!(result.is_ok());

        // Try to verify with a different message
        let wrong_message = b"wrong message".to_vec();
        let result = verify(&wrong_message, &signature, &public_key);
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

    #[test]
    fn test_vec_to_signature() {
        // Valid case
        let valid_bytes = vec![0u8; 64];
        let result = to_signature_bytes(&valid_bytes);
        assert!(result.is_ok());

        // Invalid case - wrong length
        let invalid_bytes = vec![0u8; 63];
        let result = to_signature_bytes(&invalid_bytes);
        assert!(result.is_err());
    }
}
