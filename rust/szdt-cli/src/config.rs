use crate::error::Error;
use std::path::PathBuf;

/// Returns the path to the SZDT config directory.
pub fn config_dir() -> Result<PathBuf, Error> {
    Ok(dirs::home_dir()
        .ok_or(Error::Fs("Unable to locate home directory".to_string()))?
        .join(".szdt"))
}

/// Returns the path to the keys directory.
pub fn contacts_file() -> Result<PathBuf, Error> {
    Ok(config_dir()?.join("contacts.sqlite"))
}
