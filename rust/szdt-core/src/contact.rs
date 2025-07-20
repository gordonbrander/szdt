use crate::did::DidKey;
use crate::ed25519_key_material::Ed25519KeyMaterial;
use crate::error::Error;
use crate::nickname::Nickname;

#[derive(Debug)]
pub struct Contact {
    pub nickname: Nickname,
    pub did: DidKey,
    pub private_key: Option<Vec<u8>>,
}

impl Contact {
    pub fn new(nickname: Nickname, did: DidKey, private_key: Option<Vec<u8>>) -> Self {
        Contact {
            nickname,
            did,
            private_key,
        }
    }
}

impl TryFrom<&Contact> for Ed25519KeyMaterial {
    type Error = Error;

    fn try_from(contact: &Contact) -> Result<Self, Self::Error> {
        if let Some(private_key) = &contact.private_key {
            let key_material = Ed25519KeyMaterial::try_from_private_key(private_key)?;
            return Ok(key_material);
        }
        let key_material = Ed25519KeyMaterial::try_from(&contact.did)?;
        Ok(key_material)
    }
}

impl From<&Contact> for DidKey {
    fn from(contact: &Contact) -> Self {
        contact.did.clone()
    }
}
