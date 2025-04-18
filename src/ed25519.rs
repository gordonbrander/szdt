use crate::error::{Error, Result};
pub use ed25519_dalek::SecretKey;
use ed25519_dalek::{PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, Signature, SigningKey, VerifyingKey};
use ed25519_dalek::{Signer, Verifier};
use rand::rngs::OsRng;

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
pub fn vec_to_public_key(bytes: &Vec<u8>) -> Result<PublicKey> {
    if bytes.len() != PUBLIC_KEY_LENGTH {
        return Err(Error::ValueError(format!(
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
pub fn sign(bytes: &Vec<u8>, secret_key: &SecretKey) -> SignatureBytes {
    // Generate a keypair
    let signing_key = SigningKey::from_bytes(secret_key);
    signing_key.sign(&bytes).to_bytes()
}

/// Generate a new private key
pub fn generate_private_key() -> SecretKey {
    let mut csprng = OsRng;
    SigningKey::generate(&mut csprng).to_bytes()
}

/// Convert a Vec<u8> to PrivateKey.
/// Returns an error if the input is not exactly 32 bytes.
pub fn vec_to_private_key(bytes: &Vec<u8>) -> Result<SecretKey> {
    if bytes.len() != SECRET_KEY_LENGTH {
        return Err(Error::ValueError(format!(
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
pub fn vec_to_signature(bytes: &Vec<u8>) -> Result<SignatureBytes> {
    if bytes.len() != 64 {
        return Err(Error::ValueError(format!(
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
) -> Result<()> {
    let verifying_key = VerifyingKey::from_bytes(public_key)?;
    let signature = Signature::from_bytes(signature_bytes);
    // Verify the signature
    match verifying_key.verify(bytes, &signature) {
        Ok(()) => Ok(()),
        Err(_) => Err(Error::SignatureVerificationError(
            "Signature didn't verify".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation_and_signing() {
        // Generate a private key
        let secret_key = generate_private_key();

        // Get the corresponding public key
        let public_key = get_public_key(&secret_key);

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
        let result = vec_to_public_key(&valid_bytes);
        assert!(result.is_ok());

        // Invalid case - wrong length
        let invalid_bytes = vec![0u8; PUBLIC_KEY_LENGTH - 1];
        let result = vec_to_public_key(&invalid_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_vec_to_private_key() {
        // Valid case
        let valid_bytes = vec![0u8; SECRET_KEY_LENGTH];
        let result = vec_to_private_key(&valid_bytes);
        assert!(result.is_ok());

        // Invalid case - wrong length
        let invalid_bytes = vec![0u8; SECRET_KEY_LENGTH - 1];
        let result = vec_to_private_key(&invalid_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_vec_to_signature() {
        // Valid case
        let valid_bytes = vec![0u8; 64];
        let result = vec_to_signature(&valid_bytes);
        assert!(result.is_ok());

        // Invalid case - wrong length
        let invalid_bytes = vec![0u8; 63];
        let result = vec_to_signature(&invalid_bytes);
        assert!(result.is_err());
    }
}
