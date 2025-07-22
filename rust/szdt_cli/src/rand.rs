use crate::error::Error;
use rand::{TryRngCore, rngs::OsRng};

pub fn generate_entropy() -> Result<[u8; 32], Error> {
    let mut rng = OsRng;
    let mut bytes = [0u8; 32];
    rng.try_fill_bytes(&mut bytes)
        .map_err(|e| Error::Rand(e.to_string()))?;
    Ok(bytes)
}
