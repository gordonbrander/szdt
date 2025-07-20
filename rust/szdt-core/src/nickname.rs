use crate::error::Error;
use crate::text::truncate;
use thiserror::Error;

const NICKNAME_MAX_LENGTH: usize = 63;

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nickname(String);

impl Nickname {
    /// Parses a string into a valid nickname.
    /// Note that this is a lossy process.
    pub fn parse(text: &str) -> Result<Nickname, NicknameError> {
        let mut nickname: String = text
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .take(NICKNAME_MAX_LENGTH)
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

    pub fn with_suffix(text: &str, suffix: &str) -> Result<Nickname, Error> {
        let suffix_len = suffix.chars().count();
        let truncated = truncate(text, NICKNAME_MAX_LENGTH - suffix_len, "");
        let text_with_suffix = format!("{}{}", truncated, suffix);
        let nickname = Self::parse(&text_with_suffix)?;
        Ok(nickname)
    }

    /// Borrow nickname as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Nickname {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&str> for Nickname {
    type Error = NicknameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl TryFrom<String> for Nickname {
    type Error = NicknameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

#[derive(Debug, Error)]
pub enum NicknameError {
    #[error("Nickname is too short. Must be at least 1 character.")]
    TooShort,
}
