use crate::db::migrations::migrate;
use crate::error::Error;
use rusqlite::params;
use std::path::Path;
use szdt_core::contact::Contact;
use szdt_core::did::DidKey;
use szdt_core::nickname::Nickname;

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

fn contact_from_table_row(row: &rusqlite::Row) -> Result<Contact, Error> {
    let nickname_string: String = row.get(0)?;
    let nickname = Nickname::try_from(nickname_string)?;
    let did_url_string: String = row.get(1)?;
    let did = DidKey::try_from(did_url_string)?;
    let private_key = row.get(2)?;
    Ok(Contact::new(nickname, did, private_key))
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
    pub fn contact(&self, nickname: &Nickname) -> Result<Option<Contact>, Error> {
        match self.db.query_row_and_then(
            "SELECT nickname, did, private_key FROM contact WHERE nickname = ?",
            params![nickname.to_string()],
            contact_from_table_row,
        ) {
            Ok(contact) => Ok(Some(contact)),
            Err(Error::Sqlite(rusqlite::Error::QueryReturnedNoRows)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn contact_for_did(&self, did: &DidKey) -> Result<Option<Contact>, Error> {
        let did_string = did.to_string();
        match self.db.query_row_and_then(
            "SELECT nickname, did, private_key FROM contact WHERE did = ?",
            [did_string],
            contact_from_table_row,
        ) {
            Ok(contact) => Ok(Some(contact)),
            Err(Error::Sqlite(rusqlite::Error::QueryReturnedNoRows)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Generate a unique nickname for a new contact. If nickname given has
    /// not been taken, will just return it. Otherwise, will attempt to make it
    /// unique by appending a random suffix.
    pub fn unique_nickname(&self, text: &str) -> Result<Nickname, Error> {
        let default_nickname = Nickname::parse("anon")?;
        let nickname = Nickname::parse(text).unwrap_or(default_nickname);

        if self.contact(&nickname)?.is_none() {
            return Ok(nickname.clone());
        }

        for i in 0..128 {
            // We start at 2 so we get "foo", "foo2", "foo3", etc.
            let suffix = (i + 2).to_string();
            let draft_nickname = Nickname::with_suffix(nickname.to_string().as_str(), &suffix)?;
            if self.contact(&draft_nickname)?.is_none() {
                return Ok(draft_nickname);
            }
        }

        Err(Error::NicknameAlreadyTaken(
            "Nickname is already taken. Unable to make it unique.".to_string(),
        ))
    }

    /// Create a new public/private keypair, stored at nickname.
    /// Nickname must be unique. If a record with this nickname already exists,
    /// a Sqlite error will be returned.
    pub fn create_contact(&self, contact: &Contact) -> Result<(), Error> {
        self.db.execute(
            "INSERT INTO contact (nickname, did, private_key) VALUES (?, ?, ?)",
            params![
                contact.nickname.to_string(),
                &contact.did.to_string(),
                &contact.private_key,
            ],
        )?;
        Ok(())
    }

    pub fn delete_contact(&self, nickname: &str) -> Result<(), Error> {
        self.db
            .execute("DELETE FROM contact WHERE nickname = ?", params![nickname])?;
        Ok(())
    }

    /// Get a HashMap of key nicknames to DID
    pub fn contacts(&self) -> Result<Vec<Contact>, Error> {
        let mut stmt = self
            .db
            .prepare("SELECT nickname, did, private_key FROM contact ORDER BY nickname")?;
        let mut contacts: Vec<Contact> = Vec::new();
        for contact in stmt.query_and_then([], contact_from_table_row)? {
            contacts.push(contact?);
        }
        Ok(contacts)
    }
}
