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
pub fn sign(bytes: &Vec<u8>, secret_key: &SecretKey) -> Vec<u8> {
    // Generate a keypair
    let signing_key = SigningKey::from_bytes(secret_key);
    signing_key.sign(&bytes).to_vec()
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
