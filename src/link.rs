use crate::error::Error;
use crate::hash::Hash;

pub trait IntoLink {
    /// Generate a Blake3 hash representing the ipld-dagcbor serialized data.
    fn into_link(&self) -> Result<Hash, Error>;
}

impl<T> IntoLink for T
where
    T: serde::Serialize,
{
    fn into_link(&self) -> Result<Hash, Error> {
        let bytes = serde_ipld_dagcbor::to_vec(self)?;
        Ok(Hash::new(bytes))
    }
}
