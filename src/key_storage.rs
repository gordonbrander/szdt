use crate::did::DidKey;
use crate::ed25519_key_material::Ed25519KeyMaterial;
use crate::error::Error;
use crate::file::{list_files, write_file_deep};
use crate::mnemonic::Mnemonic;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

pub struct InsecureKeyStorage {
    key_storage_dir: PathBuf,
}

impl InsecureKeyStorage {
    pub fn new(key_storage_dir: PathBuf) -> Result<Self, Error> {
        Ok(InsecureKeyStorage { key_storage_dir })
    }

    pub fn key_exists(&self, nickname: &str) -> bool {
        self.public_key_path(nickname).exists() || self.private_key_path(nickname).exists()
    }

    fn private_key_path(&self, nickname: &str) -> PathBuf {
        self.key_storage_dir
            .join(nickname)
            .with_extension("private")
    }

    fn public_key_path(&self, nickname: &str) -> PathBuf {
        self.key_storage_dir.join(nickname).with_extension("public")
    }

    /// Read key with name, returning Ed25519KeyMaterial with private key
    pub fn key(&self, nickname: &str) -> Result<Option<Ed25519KeyMaterial>, Error> {
        let private_key_path = self.private_key_path(nickname);
        // Load from private key if it exists
        if private_key_path.exists() {
            let mnemonic_string = fs::read_to_string(private_key_path)?;
            let mnemonic = Mnemonic::parse(&mnemonic_string)?;
            let key_material = Ed25519KeyMaterial::try_from(&mnemonic)?;
            return Ok(Some(key_material));
        }
        // Load from public key if it exists (this key material can't sign)
        let public_key_path = self.public_key_path(nickname);
        if public_key_path.exists() {
            let did_string = fs::read_to_string(public_key_path)?;
            let did = DidKey::try_from(did_string.as_str())?;
            let key_material = Ed25519KeyMaterial::try_from(&did)?;
            return Ok(Some(key_material));
        }
        // Otherwise, no key exists by this name
        Ok(None)
    }

    /// Register a key under a nickname
    pub fn create_key(
        &self,
        nickname: &str,
        key_material: &Ed25519KeyMaterial,
    ) -> Result<(), Error> {
        // Don't overwrite existing keys
        if self.key_exists(nickname) {
            return Err(Error::KeyExists(nickname.to_string()));
        }

        // Write the public key
        let did = DidKey::from(key_material);
        write_file_deep(self.public_key_path(nickname), did.to_string())?;

        // Write the private key (if it exists)
        match Mnemonic::try_from(key_material) {
            Ok(mnemonic) => {
                write_file_deep(self.private_key_path(nickname), mnemonic.to_string())?;
                Ok(())
            }
            Err(Error::PrivateKeyMissing(_)) => Ok(()),
            Err(err) => Err(err),
        }
    }

    pub fn delete_key(&self, nickname: &str) -> Result<(), Error> {
        if !self.key_exists(nickname) {
            return Err(Error::KeyNotFound(nickname.to_string()));
        }
        let public_key_path = self.public_key_path(nickname);
        if public_key_path.exists() {
            fs::remove_file(public_key_path)?;
        }
        let private_key_path = self.private_key_path(nickname);
        if private_key_path.exists() {
            fs::remove_file(private_key_path)?;
        }
        Ok(())
    }

    /// Get a BTreeMap of key nicknames to DID
    pub fn keys(&self) -> Result<BTreeMap<String, DidKey>, Error> {
        let mut key_index = BTreeMap::new();
        let public_ext = OsStr::new("public");
        for path in list_files(&self.key_storage_dir)? {
            if path.extension() != Some(public_ext) {
                continue;
            };
            let did_string = fs::read_to_string(&path)?;
            let did = DidKey::try_from(did_string.as_str())?;
            let stem = path.file_stem().ok_or(Error::Fs(format!(
                "Could not read stem from path {}",
                path.display()
            )))?;
            let name = stem.to_string_lossy().to_string();
            key_index.insert(name, did);
        }
        Ok(key_index)
    }
}
