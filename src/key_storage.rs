use crate::db::migrations::migrate;
use crate::did::DidKey;
use crate::ed25519_key_material::Ed25519KeyMaterial;
use crate::error::Error;
use crate::nickname::Nickname;
use rusqlite::params;
use std::path::Path;

fn migration1(tx: &rusqlite::Transaction) -> Result<(), rusqlite::Error> {
    tx.execute(
        "CREATE TABLE IF NOT EXISTS contact (
            nickname TEXT PRIMARY KEY,
            did TEXT NOT NULL,
            private_key BLOB
        )",
        [],
    )?;
    Ok(())
}

pub struct InsecureKeyStorage {
    db: rusqlite::Connection,
}

impl InsecureKeyStorage {
    pub fn new(file_path: &Path) -> Result<Self, Error> {
        let mut db = rusqlite::Connection::open(file_path)?;
        migrate(&mut db, &[migration1])?;
        Ok(InsecureKeyStorage { db })
    }

    /// Read key with name, returning Ed25519KeyMaterial with private key
    pub fn key(&self, nickname: &Nickname) -> Result<Option<Ed25519KeyMaterial>, Error> {
        match self.db.query_row_and_then(
            "SELECT nickname, did, private_key FROM contact WHERE nickname = ?",
            params![nickname.to_string()],
            map_contact_row,
        ) {
            Ok((_nickname, key_material)) => Ok(Some(key_material)),
            Err(Error::Sqlite(rusqlite::Error::QueryReturnedNoRows)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn key_for_did(&self, did: &DidKey) -> Result<Option<Ed25519KeyMaterial>, Error> {
        let did_string = did.to_string();
        match self.db.query_row_and_then(
            "SELECT nickname, did, private_key FROM contact WHERE did = ?",
            [did_string],
            map_contact_row,
        ) {
            Ok((_nickname, key_material)) => Ok(Some(key_material)),
            Err(Error::Sqlite(rusqlite::Error::QueryReturnedNoRows)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Generate a unique nickname for a new contact. If nickname given has
    /// not been taken, will just return it. Otherwise, will attempt to make it
    /// unique by appending a random suffix.
    pub fn unique_nickname(&self, text: &str) -> Result<Nickname, Error> {
        let default_nickname = Nickname::parse("anon")?;
        let nickname = Nickname::parse(&text).unwrap_or(default_nickname);

        if self.key(&nickname)?.is_none() {
            return Ok(nickname.clone());
        }

        for i in 0..128 {
            let suffix = (i + 1).to_string();
            let draft_nickname = Nickname::with_suffix(nickname.to_string().as_str(), &suffix)?;
            if self.key(&draft_nickname)?.is_none() {
                return Ok(draft_nickname);
            }
        }

        Err(Error::NicknameAlreadyTaken(
            "Nickname {} is already taken. Unable to make it unique.".to_string(),
        ))
    }

    /// Create a new public/private keypair, stored at nickname.
    /// Nickname must be unique. If a record with this nickname already exists,
    /// a Sqlite error will be returned.
    pub fn create_key(
        &self,
        nickname: &Nickname,
        key_material: &Ed25519KeyMaterial,
    ) -> Result<(), Error> {
        self.db.execute(
            "INSERT INTO contact (nickname, did, private_key) VALUES (?, ?, ?)",
            params![
                nickname.to_string(),
                &key_material.did().to_string(),
                &key_material.private_key(),
            ],
        )?;
        Ok(())
    }

    pub fn get_or_create_key(&self, nickname: &Nickname) -> Result<Ed25519KeyMaterial, Error> {
        if let Some(key_material) = self.key(nickname)? {
            return Ok(key_material);
        }
        let key_material = Ed25519KeyMaterial::generate();
        self.create_key(nickname, &key_material)?;
        Ok(key_material)
    }

    pub fn delete_key(&self, nickname: &str) -> Result<(), Error> {
        self.db
            .execute("DELETE FROM contact WHERE nickname = ?", params![nickname])?;
        Ok(())
    }

    /// Get a HashMap of key nicknames to DID
    pub fn keys(&self) -> Result<Vec<(Nickname, Ed25519KeyMaterial)>, Error> {
        let mut stmt = self
            .db
            .prepare("SELECT nickname, did, private_key FROM contact ORDER BY nickname")?;
        let mut contacts: Vec<(Nickname, Ed25519KeyMaterial)> = Vec::new();
        for key_material in stmt.query_and_then([], map_contact_row)? {
            contacts.push(key_material?);
        }
        Ok(contacts)
    }
}

fn map_contact_row(row: &rusqlite::Row) -> Result<(Nickname, Ed25519KeyMaterial), Error> {
    let nickname_string: String = row.get(0)?;
    let nickname = Nickname::parse(&nickname_string)?;
    let did_url: String = row.get(1)?;
    let private_key: Option<Vec<u8>> = row.get(2)?;
    match (did_url, private_key) {
        (_did_url, Some(private_key)) => {
            let key_material = Ed25519KeyMaterial::try_from_private_key(&private_key)?;
            Ok((nickname, key_material))
        }
        (did_url, None) => {
            let did = DidKey::try_from(did_url.as_str())?;
            let key_material = Ed25519KeyMaterial::try_from(&did)?;
            Ok((nickname, key_material))
        }
    }
}
