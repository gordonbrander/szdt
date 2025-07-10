use crate::memo::Memo;
use serde::{Deserialize, Serialize};

/// Represents a value in the SZDT format.
/// Either Memo or a CBOR Value.
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum Value {
    Memo(Memo),
    Value(cbor4ii::core::Value),
}
