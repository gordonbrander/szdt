use crate::error::Error;
use data_encoding::BASE32_NOPAD;
use rand::Rng;
use thiserror::Error;

/// A nickname is a string that follows domain-name-compatible rules, less the TLD.
///
/// A domain name:
/// - Can only contain A-Z, 0-9 and hyphen (-), in addition to one punctuation (.).
///   We omit the punctuation (tld).
/// - Max length is 63 characters, excluding the extension like .com
/// - Min length is 1 character
/// - Hyphens cannot be the first or last character of the domain name.
/// - Multiple hyphens are discouraged.
///
/// We omit the TLD, so (.) is not allowed.
pub struct Nickname(String);

impl Nickname {
    /// Parses a string into a valid nickname.
    /// Note that this is a lossy process.
    pub fn parse(text: &str) -> Result<Nickname, NicknameError> {
        let mut nickname: String = text
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .take(63)
            .collect::<String>()
            .to_lowercase();

        if nickname.starts_with('-') {
            nickname.remove(0);
        }

        if nickname.ends_with('-') {
            nickname.remove(nickname.len() - 1);
        }

        if nickname.len() < 1 {
            return Err(NicknameError::TooShort);
        }

        Ok(Nickname(nickname))
    }

    /// Adds a 4-character base32 lowercase suffix.
    pub fn with_random_suffix(text: &str) -> Result<Nickname, Error> {
        let mut rng = rand::rng();
        let random_bytes: [u8; 3] = rng.random(); // 3 bytes = 24 bits, enough for 4 base32 chars
        let suffix = BASE32_NOPAD.encode(&random_bytes);
        let suffix_4char = &suffix[..4]; // Take first 4 characters
        let full_name = format!("{}{}", text, suffix_4char);
        let nickname = Self::parse(&full_name)?;
        Ok(nickname)
    }
}

impl std::fmt::Display for Nickname {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Error)]
pub enum NicknameError {
    #[error("Nickname is too short. Must be at least 1 character.")]
    TooShort,
}
