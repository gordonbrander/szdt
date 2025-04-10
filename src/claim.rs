use crate::cid::Cid;
use crate::ed25519::{SecretKey, sign};
use data_encoding;
use serde_json;

pub struct AuthorClaim {
    pub iss: String,
    pub cid: Cid,
    pub exp: Option<u64>,
    pub iat: u64,
    pub nbf: Option<u64>,
}

const JWT_ED25519_ALGORITHM: &str = "EdDSA";

impl AuthorClaim {
    /// Create a new author claim.
    pub fn new(iss: String, cid: Cid, exp: Option<u64>, iat: u64, nbf: Option<u64>) -> Self {
        AuthorClaim {
            iss,
            cid,
            exp,
            iat,
            nbf,
        }
    }

    /// Sign the author claim using the given private key.
    /// Returns the JWT token string.
    pub fn sign_jwt(&self, private_key: &SecretKey) -> String {
        let headers = serde_json::json!({
            "alg": JWT_ED25519_ALGORITHM,
        });

        let payload = serde_json::json!({
            "knd": "szdt/claim/author",
            "iss": self.iss,
            "sub": self.cid.to_string(),
            "exp": self.exp,
            "iat": self.iat,
            "nbf": self.nbf,
        });

        let encoded_headers = data_encoding::BASE64URL.encode(headers.to_string().as_bytes());
        let encoded_payload = data_encoding::BASE64URL.encode(payload.to_string().as_bytes());

        let to_sign = format!("{}.{}", encoded_headers, encoded_payload);

        let signature = sign(to_sign.as_bytes(), private_key);
        let encoded_signature = data_encoding::BASE64URL.encode(&signature);

        let token = format!(
            "{}.{}.{}",
            encoded_headers, encoded_payload, encoded_signature
        );

        token
    }
}
