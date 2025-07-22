use ed25519_dalek::{
    self, PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, SecretKey, Signature, Signer, SigningKey, Verifier,
    VerifyingKey,
};
use thiserror::Error;

pub type PublicKey = [u8; PUBLIC_KEY_LENGTH];
pub type SignatureBytes = [u8; 64];
pub type PrivateKey = [u8; SECRET_KEY_LENGTH];

/// Generate a signing keypair from 32 bytes of entropy.
/// Returns a tuple of `(pubkey, privkey)`.
pub fn generate_keypair_from_entropy(seed: &[u8]) -> Result<(PublicKey, PrivateKey), Error> {
    if seed.len() != SECRET_KEY_LENGTH {
        return Err(Error::InvalidKey(format!(
            "Seed must be {} bytes, got {}",
            SECRET_KEY_LENGTH,
            seed.len()
        )));
    }

    let mut secret_key = [0u8; SECRET_KEY_LENGTH];
    secret_key.copy_from_slice(seed);

    let signing_key = SigningKey::from_bytes(&secret_key);
    Ok((
        signing_key.verifying_key().to_bytes(),
        signing_key.to_bytes(),
    ))
}

/// Get the public key from a private key.
pub fn derive_public_key(private_key: &[u8]) -> Result<PublicKey, Error> {
    let private_key = to_private_key(private_key)?;
    let signing_key = SigningKey::from_bytes(&private_key);
    let public_key = to_public_key(&signing_key.verifying_key().to_bytes())?;
    Ok(public_key)
}

/// Sign a payload with a private key.
/// Returns the signature as a Vec<u8>.
pub fn sign(payload: &[u8], private_key: &[u8]) -> Result<Vec<u8>, Error> {
    let private_key = to_private_key(private_key)?;
    let signing_key = SigningKey::from_bytes(&private_key);
    let signature = signing_key.sign(payload);
    Ok(signature.to_bytes().to_vec())
}

/// Verify a signature with a public key.
/// Returns an error if the signature is invalid.
pub fn verify(payload: &[u8], signature: &[u8], public_key: &[u8]) -> Result<(), Error> {
    let public_key = to_public_key(public_key)?;
    let signature = Signature::from_slice(signature)?;
    let verifying_key = VerifyingKey::from_bytes(&public_key)?;
    verifying_key.verify(payload, &signature)?;
    Ok(())
}

/// Convert a Vec<u8> to PublicKey.
/// Returns an error if the input is not exactly 32 bytes.
pub fn to_public_key(bytes: &[u8]) -> Result<PublicKey, Error> {
    if bytes.len() != PUBLIC_KEY_LENGTH {
        return Err(Error::InvalidKey(format!(
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
pub fn to_private_key(bytes: &[u8]) -> Result<SecretKey, Error> {
    if bytes.len() != SECRET_KEY_LENGTH {
        return Err(Error::InvalidKey(format!(
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
    Signature(#[from] ed25519_dalek::SignatureError),
    #[error("Invalid key length: {0}")]
    InvalidKey(String),
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let result = to_private_key(&valid_bytes);
        assert!(result.is_ok());

        // Invalid case - wrong length
        let invalid_bytes = vec![0u8; SECRET_KEY_LENGTH - 1];
        let result = to_private_key(&invalid_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_derive_public_key_derives_the_public_key() {
        // Generate a keypair to test with using test entropy
        let test_seed = [1u8; SECRET_KEY_LENGTH];
        let (expected_pubkey, privkey) = generate_keypair_from_entropy(&test_seed).unwrap();

        // Derive public key from private key
        let derived_pubkey = derive_public_key(&privkey).unwrap();

        // Should match the original public key
        assert_eq!(expected_pubkey, derived_pubkey);
    }

    #[test]
    fn test_derive_public_key_returns_err_for_invalid_key() {
        // Test with invalid private key length
        let invalid_privkey = vec![0u8; SECRET_KEY_LENGTH - 1];
        let result = derive_public_key(&invalid_privkey);
        assert!(result.is_err());
    }

    #[test]
    fn test_sign_verify_roundtrip() {
        let test_seed = [2u8; SECRET_KEY_LENGTH];
        let (pubkey, privkey) = generate_keypair_from_entropy(&test_seed).unwrap();
        let payload = b"test message";

        // Valid signing
        let signature = sign(payload, &privkey).unwrap();

        // Valid verification
        let result = verify(payload, &signature, &pubkey);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sign_returns_err_for_invalid_key() {
        // Test with invalid private key length
        let invalid_privkey = vec![0u8; SECRET_KEY_LENGTH - 1];
        let result = sign(b"test message", &invalid_privkey);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_keypair_from_seed_valid() {
        let test_seed = [42u8; SECRET_KEY_LENGTH];
        let result = generate_keypair_from_entropy(&test_seed);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_keypair_from_seed_invalid_length() {
        let invalid_seed = vec![0u8; SECRET_KEY_LENGTH - 1];
        let result = generate_keypair_from_entropy(&invalid_seed);
        assert!(result.is_err());
    }
}
