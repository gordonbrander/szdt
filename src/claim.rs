use crate::cid::Cid;
use crate::did::decode_ed25519_did_key;
use crate::ed25519::{SecretKey, sign, vec_to_signature, verify};
use crate::error::{Error, Result};
use data_encoding;
use serde_json::{self, Value};

pub struct AuthorClaim {
    pub iss: String,
    pub cid: Cid,
    pub exp: Option<u64>,
    pub iat: u64,
    pub nbf: Option<u64>,
}

const JWT_ED25519_ALGORITHM: &str = "EdDSA";
const KND_AUTHOR: &str = "szdt/author";

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

    pub fn parse(jwt: &str) -> Result<Self> {
        let parts = jwt.splitn(3, '.').collect::<Vec<&str>>();

        let headers = data_encoding::BASE64URL.decode(parts[0].as_bytes())?;
        let payload = data_encoding::BASE64URL.decode(parts[1].as_bytes())?;
        let signature_vec = &data_encoding::BASE64URL.decode(parts[2].as_bytes())?;
        let signature = vec_to_signature(signature_vec)?;

        let Value::Object(headers) = serde_json::from_slice(&headers)? else {
            return Err(Error::ValueError(
                "JWT headers must be a JSON object.".to_string(),
            ));
        };

        if headers.get("knd").and_then(|v| v.as_str()) != Some(KND_AUTHOR) {
            return Err(Error::ValueError(
                "`knd` header is not `szdt/author`".to_string(),
            ));
        }

        let Value::Object(payload) = serde_json::from_slice(&payload)? else {
            return Err(Error::ValueError(
                "JWT payload must be a JSON object.".to_string(),
            ));
        };

        let iss = payload
            .get("iss")
            .and_then(|v| v.as_str())
            .ok_or(Error::ValueError("JWT missing iss".to_string()))?;

        // Get public key from did:key
        let public_key = decode_ed25519_did_key(iss)?;

        let cid = Cid::from_cid_str(
            payload
                .get("sub")
                .and_then(|v| v.as_str())
                .ok_or(Error::ValueError("JWT missing sub".to_string()))?,
        )?;
        let iat = payload
            .get("iat")
            .and_then(|v| v.as_u64())
            .ok_or(Error::ValueError("JWT missing iat".to_string()))?;
        let exp = payload.get("exp").and_then(|v| v.as_u64());
        let nbf = payload.get("nbf").and_then(|v| v.as_u64());

        // Verify signature
        verify(&cid.to_bytes(), &signature, &public_key)?;

        Ok(AuthorClaim {
            iss: iss.to_string(),
            cid,
            exp,
            iat,
            nbf,
        })
    }

    /// Sign the author claim using the given private key.
    /// Returns the JWT token string.
    pub fn sign_jwt(&self, private_key: &SecretKey) -> String {
        let headers = serde_json::json!({
            "alg": JWT_ED25519_ALGORITHM,
            "knd": KND_AUTHOR,
        });

        let payload = serde_json::json!({
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
