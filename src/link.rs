use crate::error::Error;
use crate::hash::Hash;
use serde::Serialize;
use serde_ipld_dagcbor;

pub trait ToLink {
    fn to_link(&self) -> Result<Hash, Error>;
}

impl<T> ToLink for T
where
    T: Serialize,
{
    /// Generate a content-addressed link from the given data.
    /// Serializes content to CBOR and generates a Blake3 hash for that CBOR data.
    fn to_link(&self) -> Result<Hash, Error> {
        let cbor_data = serde_ipld_dagcbor::to_vec(self)?;
        let hash = Hash::new(&cbor_data);
        Ok(hash)
    }
}
