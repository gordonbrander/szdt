use crate::db::migrations::MigrationError;
use szdt_core::error::Error as CoreError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Core error: {0}")]
    Core(#[from] CoreError),
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Migration error: {0}")]
    MigrationError(#[from] MigrationError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Unable generate randomness: {0}")]
    Rand(String),
    #[error("Error stripping path prefix")]
    StripPrefix(#[from] std::path::StripPrefixError),
    #[error("File system error: {0}")]
    Fs(String),
    #[error("Nickname already taken: {0}")]
    NicknameAlreadyTaken(String),
}

impl From<szdt_core::nickname::NicknameError> for Error {
    fn from(err: szdt_core::nickname::NicknameError) -> Self {
        Error::Core(err.into())
    }
}

impl From<szdt_core::did::Error> for Error {
    fn from(err: szdt_core::did::Error) -> Self {
        Error::Core(err.into())
    }
}
