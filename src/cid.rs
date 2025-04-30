use crate::multiformats::{MULTICODEC_DCBOR, MULTICODEC_RAW};
use crate::multihash::{self, read_into_multihash};
pub use cid::Cid;
use std::io::{self, Read};
use thiserror::Error;

/// Read bytes into a CID v1.
pub fn read_into_cid_v1<R: Read>(codec: u64, reader: &mut R) -> Result<Cid, Error> {
    let hash = read_into_multihash(reader)?;
    Ok(Cid::new_v1(codec, hash))
}

/// Read bytes into a CID v1 with a raw codec
pub fn read_into_cid_v1_raw<R: Read>(reader: &mut R) -> Result<Cid, Error> {
    read_into_cid_v1(MULTICODEC_RAW, reader)
}

/// Read bytes into a CID v1 with a dag-cbor codec
pub fn read_into_cid_v1_cbor<R: Read>(reader: &mut R) -> Result<Cid, Error> {
    read_into_cid_v1(MULTICODEC_DCBOR, reader)
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error")]
    Io(#[from] io::Error),
    #[error("Multihash error")]
    Multihash(multihash::Error),
}

impl From<multihash::Error> for Error {
    fn from(err: multihash::Error) -> Self {
        match err {
            multihash::Error::Io(err) => Error::Io(err),
            _ => Error::Multihash(err),
        }
    }
}
