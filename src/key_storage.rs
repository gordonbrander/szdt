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
        if !private_key_path.exists() {
            return Ok(None);
        }
        let mnemonic_string = fs::read_to_string(private_key_path)?;
        let mnemonic = Mnemonic::parse(&mnemonic_string)?;
        let key_material = Ed25519KeyMaterial::try_from(&mnemonic)?;
        Ok(Some(key_material))
    }

    pub fn create_key(&self, nickname: &str) -> Result<Ed25519KeyMaterial, Error> {
        if let Some(key_material) = self.key(nickname)? {
            return Ok(key_material);
        }
        let key_material = Ed25519KeyMaterial::generate();
        let mnemonic = Mnemonic::try_from(&key_material)?;
        let did = DidKey::from(&key_material);
        write_file_deep(self.private_key_path(nickname), mnemonic.to_string())?;
        write_file_deep(self.public_key_path(nickname), did.to_string())?;
        Ok(key_material)
    }

    pub fn delete_key(&self, nickname: &str) -> Result<(), Error> {
        let private_key_path = self.private_key_path(nickname);
        let public_key_path = self.public_key_path(nickname);
        if !private_key_path.exists() || !public_key_path.exists() {
            return Err(Error::Fs(format!("Key not found: {}", nickname)));
        }
        fs::remove_file(private_key_path)?;
        fs::remove_file(public_key_path)?;
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
