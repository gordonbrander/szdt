use crate::varint::{self, read_varint_usize};
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use std::io::{self, Read, Write};

pub const MULTIBASE_BASE32: &str = "b";
pub const MULTIBASE_BASE2: usize = 0;
pub const CID_VERSION: usize = 1;
pub const MULTICODEC_RAW: usize = 0x55;
pub const MULTICODEC_DAG_CBOR: usize = 0x71;
pub const MULTIHASH_SHA256: usize = 0x12;
pub const SHA256_DIGEST_LENGTH: usize = 32;

/// CID v1 as specified in DASL.
/// See <https://dasl.ing/cid.html>.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DaslCid {
    codec: Codec,
    digest: [u8; 32],
}

impl DaslCid {
    /// Create a new CID v1 from a codec and a SHA256 digest.
    pub fn new(codec: Codec, digest: &[u8]) -> Self {
        let mut digest_array = [0u8; 32];
        digest_array.copy_from_slice(digest);
        Self {
            codec,
            digest: digest_array,
        }
    }

    /// Read and streaming hash the raw bytes of a reader, generating a CID for those bytes
    pub fn hash<R: Read>(reader: &mut R, codec: Codec) -> Result<Self, Error> {
        let mut hasher = Sha256::new();
        // Streaming hash the raw bytes
        io::copy(reader, &mut hasher)?;
        let digest = hasher.finalize();
        Ok(Self::new(codec, &digest.as_slice()))
    }

    /// Read a binary CID v1 from a reader.
    /// Supports CID types specified in <https://dasl.ing/cid.html>.
    pub fn read_cid<R: Read>(reader: &mut R) -> Result<Self, Error> {
        // CID byte structure at this point:
        // <multibase><version><multicodec><multihash><length><digest>

        // Read multibase prefix
        let multibase = read_varint_usize(reader)?;
        if multibase != MULTIBASE_BASE2 {
            return Err(Error::UnsupportedMultibase(format!("{}", multibase)));
        }

        // Read CID body
        Self::read_cid_body(reader)
    }

    /// Read the body portion of a CID v1 (e.g. the portion without the multibase prefix)
    fn read_cid_body<R: Read>(reader: &mut R) -> Result<Self, Error> {
        // Remaining CID byte structure:
        // <version><multicodec><multihash><length><digest>

        // Check that version is 1
        let cid_version = read_varint_usize(reader)?;
        if cid_version != CID_VERSION {
            return Err(Error::UnsupportedVersion(format!("{}", cid_version)));
        }

        // Parse codec
        let multicodec = read_varint_usize(reader)?;
        let codec: Codec = multicodec.try_into()?;

        // Check multihash. We only support SHA256.
        let multihash = read_varint_usize(reader)?;
        if multihash != MULTIHASH_SHA256 {
            return Err(Error::UnsupportedHash(format!("{}", multihash)));
        }

        // Parse digest
        let digest_len = read_varint_usize(reader)?;
        if digest_len != SHA256_DIGEST_LENGTH {
            return Err(Error::Other("Wrong digest length for SHA256".to_string()));
        }
        let mut digest = [0; SHA256_DIGEST_LENGTH];
        reader.read_exact(&mut digest)?;

        Ok(DaslCid { codec, digest })
    }

    /// Write binary CID to writer
    /// Writes the CID in binary CID format, including the base2 multibase prefix.
    /// Returns the number of bytes written.
    pub fn write_cid<W: Write>(&self, writer: &mut W) -> Result<usize, Error> {
        let mut written = 0;
        // Write multibase prefix
        written += varint::write_usize_varint(writer, MULTIBASE_BASE2)?;
        // Write CID version
        written += varint::write_usize_varint(writer, CID_VERSION)?;
        // Write multicodec
        written += varint::write_usize_varint(writer, self.codec as usize)?;
        // Write multihash
        written += varint::write_usize_varint(writer, MULTIHASH_SHA256)?;
        // Write digest length
        written += varint::write_usize_varint(writer, SHA256_DIGEST_LENGTH)?;
        // Write digest
        writer.write_all(&self.digest)?;
        Ok(written + &self.digest.len())
    }

    pub fn version(&self) -> usize {
        CID_VERSION
    }

    pub fn codec(&self) -> Codec {
        self.codec
    }

    pub fn multicodec(&self) -> usize {
        self.codec as usize
    }

    pub fn multihash(&self) -> usize {
        MULTIHASH_SHA256
    }

    pub fn digest(&self) -> &[u8] {
        &self.digest
    }
}

impl TryFrom<&str> for DaslCid {
    type Error = Error;

    /// Try to parse str into CID
    fn try_from(cid: &str) -> Result<Self, Self::Error> {
        if !cid.starts_with(MULTIBASE_BASE32) {
            return Err(Error::UnsupportedMultibase(cid[0..1].to_string()));
        }
        let cid_body = &cid[1..];
        let cid_body_bytes = data_encoding::BASE32_NOPAD_NOCASE.decode(cid_body.as_bytes())?;
        // Read CID body
        Self::read_cid_body(&mut cid_body_bytes.as_slice())
    }
}

impl From<&DaslCid> for String {
    /// Convert CID to string
    fn from(cid: &DaslCid) -> Self {
        // <multibase><version><multicodec><multihash><length><digest>
        let mut buf = Vec::new();
        buf.extend_from_slice(MULTIBASE_BASE32.as_bytes());
        varint::write_usize_varint(&mut buf, CID_VERSION)
            .expect("should be able to write version to buffer");
        varint::write_usize_varint(&mut buf, usize::from(cid.codec))
            .expect("should be able to write codec to buffer");
        varint::write_usize_varint(&mut buf, MULTIHASH_SHA256)
            .expect("should be able to write multihash to buffer");
        varint::write_usize_varint(&mut buf, SHA256_DIGEST_LENGTH)
            .expect("Should be able to write digest length to buffer");
        buf.extend_from_slice(&cid.digest);
        data_encoding::BASE32_NOPAD_NOCASE.encode(&buf)
    }
}

impl From<&DaslCid> for Vec<u8> {
    /// Convert CID to byte array
    fn from(cid: &DaslCid) -> Self {
        let mut buf = Vec::new();
        cid.write_cid(&mut buf)
            .expect("Should be able to write CID to buffer");
        buf
    }
}

impl std::fmt::Display for DaslCid {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

struct CidVisitor;

// Serde visitor for CID deserialization
impl<'de> serde::de::Visitor<'de> for CidVisitor {
    type Value = DaslCid;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string or byte array representing a CID v1")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        DaslCid::try_from(v).map_err(|e| E::custom(format!("Error parsing CID from string: {}", e)))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(v)
    }

    fn visit_bytes<E>(self, mut bytes: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        DaslCid::read_cid(&mut bytes)
            .map_err(|e| E::custom(format!("Error reading CID from bytes: {}", e)))
    }

    fn visit_borrowed_bytes<E>(self, bytes: &'de [u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_bytes(bytes)
    }
}

impl Serialize for DaslCid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let cid_string = String::try_from(self).map_err(|e| {
            serde::ser::Error::custom(format!("Error converting CID to string: {}", e))
        })?;
        serializer.serialize_str(&cid_string)
    }
}

impl<'de> Deserialize<'de> for DaslCid {
    fn deserialize<D>(deserializer: D) -> Result<DaslCid, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(CidVisitor)
    }
}

/// Supported codecs for CIDv1.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum Codec {
    Raw = MULTICODEC_RAW,
    DagCbor = MULTICODEC_DAG_CBOR,
}

impl TryFrom<usize> for Codec {
    type Error = Error;

    fn try_from(multicodec: usize) -> Result<Self, Self::Error> {
        match multicodec {
            MULTICODEC_RAW => Ok(Codec::Raw),
            MULTICODEC_DAG_CBOR => Ok(Codec::DagCbor),
            _ => Err(Error::UnsupportedCodec(format!("{}", multicodec))),
        }
    }
}

impl From<Codec> for usize {
    fn from(codec: Codec) -> usize {
        match codec {
            Codec::Raw => MULTICODEC_RAW,
            Codec::DagCbor => MULTICODEC_DAG_CBOR,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    BaseDecode(data_encoding::DecodeError),
    UnsignedVarIntDecode(unsigned_varint::decode::Error),
    UnsupportedMultibase(String),
    UnsupportedVersion(String),
    UnsupportedCodec(String),
    UnsupportedHash(String),
    Other(String),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            Error::UnsignedVarIntDecode(err) => Some(err),
            _ => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Io(err) => write!(f, "I/O error: {}", err),
            Error::UnsignedVarIntDecode(err) => {
                write!(f, "unsigned-varint decoding error: {}", err)
            }
            Error::BaseDecode(err) => write!(f, "Base decoding error: {}", err),
            Error::UnsupportedMultibase(base) => write!(f, "Unsupported base: {}", base),
            Error::UnsupportedVersion(version) => write!(f, "Unsupported version: {}", version),
            Error::UnsupportedCodec(codec) => write!(f, "Unsupported codec: {}", codec),
            Error::UnsupportedHash(hash) => write!(f, "Unsupported hash: {}", hash),
            Error::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<data_encoding::DecodeError> for Error {
    fn from(err: data_encoding::DecodeError) -> Self {
        Error::BaseDecode(err)
    }
}

impl From<varint::Error> for Error {
    fn from(err: varint::Error) -> Self {
        match err {
            varint::Error::Io(err) => Error::Io(err),
            varint::Error::UnsignedVarIntDecode(err) => Error::UnsignedVarIntDecode(err),
            varint::Error::Other(msg) => Error::Other(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_codec_parse() {
        assert_eq!(Codec::try_from(MULTICODEC_RAW).unwrap(), Codec::Raw);
        assert_eq!(
            Codec::try_from(MULTICODEC_DAG_CBOR).unwrap(),
            Codec::DagCbor
        );
        assert!(Codec::try_from(999).is_err());
    }

    #[test]
    fn test_read_binary_cid() {
        // Prepare a valid CIDv1 with Raw codec and SHA256 hash
        let cid_bytes = [
            0x00, // multibase (binary)
            0x01, // CID version 1
            0x55, // multicodec (raw)
            0x12, // multihash (sha256)
            0x20, // digest length (32 bytes)
            // 32 bytes of dummy digest
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
            0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c,
            0x1d, 0x1e, 0x1f, 0x20,
        ];

        let mut reader = Cursor::new(cid_bytes);
        let cid = DaslCid::read_cid(&mut reader).unwrap();

        assert_eq!(cid.codec, Codec::Raw);
        assert_eq!(
            cid.digest,
            [
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
                0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c,
                0x1d, 0x1e, 0x1f, 0x20,
            ]
        );
    }

    #[test]
    fn test_parse_str_cid() {
        // Create a base32 encoded CID string
        // Known value test - this hash is for raw "hello world"
        let cid_str = "bafkreifzjut3te2nhyekklss27nh3k72ysco7y32koao5eei66wof36n5e";

        let cid = DaslCid::try_from(cid_str).unwrap();

        // Verify it was correctly parsed
        assert_eq!(cid.codec, Codec::Raw);

        // Test invalid base prefix
        let invalid_base = "cafybeiczsscdsbs7ffqz55asqdf3smv6klcw3gofszvwlyarci47bgf354";
        assert!(DaslCid::try_from(invalid_base).is_err());
    }

    #[test]
    fn test_error_cases() {
        // Test invalid multibase
        let invalid_base = [0x01, 0x01, 0x55, 0x12, 0x20];
        let mut reader = Cursor::new(invalid_base);
        let result = DaslCid::read_cid(&mut reader);
        assert!(matches!(result, Err(Error::UnsupportedMultibase(_))));

        // Test invalid version
        let invalid_version = [0x00, 0x02, 0x55, 0x12, 0x20];
        let mut reader = Cursor::new(invalid_version);
        let result = DaslCid::read_cid(&mut reader);
        assert!(matches!(result, Err(Error::UnsupportedVersion(_))));

        // Test invalid codec
        let invalid_codec = [0x00, 0x01, 0x99, 0x12, 0x20];
        let mut reader = Cursor::new(invalid_codec);
        let result = DaslCid::read_cid(&mut reader);
        assert!(matches!(result, Err(Error::UnsupportedCodec(_))));

        // Test invalid hash algorithm
        let invalid_hash = [0x00, 0x01, 0x55, 0x13, 0x20];
        let mut reader = Cursor::new(invalid_hash);
        let result = DaslCid::read_cid(&mut reader);
        assert!(matches!(result, Err(Error::UnsupportedHash(_))));

        // Test invalid digest length
        let invalid_digest_len = [0x00, 0x01, 0x55, 0x12, 0x10];
        let mut reader = Cursor::new(invalid_digest_len);
        let result = DaslCid::read_cid(&mut reader);
        assert!(matches!(result, Err(Error::Other(_))));
    }

    #[test]
    fn test_cid_deserialize_from_str() {
        use serde_json::json;

        // Test deserializing from string
        let cid_str = "bafkreifzjut3te2nhyekklss27nh3k72ysco7y32koao5eei66wof36n5e";
        let json_str = json!(cid_str);
        let cid: DaslCid = serde_json::from_value(json_str).unwrap();
        assert_eq!(cid.codec, Codec::Raw);
    }

    #[test]
    fn test_cid_serialize_to_bytes() {
        // Create a CID with known values
        let digest = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
            0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c,
            0x1d, 0x1e, 0x1f, 0x20,
        ];
        let cid = DaslCid::new(Codec::Raw, &digest);

        // Convert to bytes
        let bytes: Vec<u8> = (&cid).into();

        // Expected byte structure:
        // <multibase=0><version=1><multicodec=0x55><multihash=0x12><length=0x20><digest>
        let expected = [
            0x00, // multibase (binary)
            0x01, // CID version 1
            0x55, // multicodec (raw)
            0x12, // multihash (sha256)
            0x20, // digest length (32 bytes)
            // 32 bytes of digest
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
            0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c,
            0x1d, 0x1e, 0x1f, 0x20,
        ];

        assert_eq!(bytes, expected);

        // Roundtrip test - ensure we can read the bytes back
        let mut reader = Cursor::new(&bytes);
        let roundtrip_cid = DaslCid::read_cid(&mut reader).unwrap();
        assert_eq!(roundtrip_cid, cid);
    }

    #[test]
    fn test_cid_serialize_dag_cbor() {
        // Create a CID with DagCbor codec
        let digest = [0x42; 32]; // Fill with arbitrary data
        let cid = DaslCid::new(Codec::DagCbor, &digest);

        // Convert to bytes
        let bytes: Vec<u8> = (&cid).into();

        // Check multicodec is correct for DagCbor
        assert_eq!(bytes[2], 0x71); // MULTICODEC_DAG_CBOR = 0x71

        // Read it back and verify codec
        let mut reader = Cursor::new(&bytes);
        let roundtrip_cid = DaslCid::read_cid(&mut reader).unwrap();
        assert_eq!(roundtrip_cid.codec(), Codec::DagCbor);
    }
}
