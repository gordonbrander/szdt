use bs58;

/// Encode bytes using Base58BTC encoding.
pub fn encode<I>(bytes: I) -> String
where
    I: AsRef<[u8]>,
{
    bs58::encode(bytes).into_string()
}

pub type DecodeError = bs58::decode::Error;

/// Decode bytes from Base58BTC encoding.
pub fn decode(s: &str) -> Result<Vec<u8>, DecodeError> {
    let bytes = bs58::decode(s).into_vec()?;
    Ok(bytes)
}
