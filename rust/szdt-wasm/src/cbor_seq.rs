use crate::memo::Memo;
use cbor4ii::core::Value;
use std::io::Cursor;
use szdt_core::cbor_seq::{CborSeqReader as CoreCborSeqReader, CborSeqWriter as CoreCborSeqWriter};
use szdt_core::error::Error as CoreError;
use wasm_bindgen::prelude::*;

/// WASM wrapper for reading CBOR sequences
#[wasm_bindgen]
pub struct CborSeqReader {
    // Store the data and current position for JavaScript compatibility
    data: Vec<u8>,
    position: usize,
}

#[wasm_bindgen]
impl CborSeqReader {
    /// Create a new CBOR sequence reader from data
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8]) -> CborSeqReader {
        Self {
            data: data.to_vec(),
            position: 0,
        }
    }

    /// Read the next memo from the sequence
    #[wasm_bindgen]
    pub fn read_memo(&mut self) -> Result<Memo, JsError> {
        if self.position >= self.data.len() {
            return Err(JsError::new("End of sequence reached"));
        }

        let remaining_data = &self.data[self.position..];
        let cursor = Cursor::new(remaining_data);
        let mut reader = CoreCborSeqReader::new(cursor);

        let core_memo: szdt_core::memo::Memo = reader.read_block().map_err(|e| match e {
            CoreError::Eof => JsError::new("End of sequence reached"),
            _ => JsError::new(&e.to_string()),
        })?;

        // Update position - we need to track how many bytes were consumed
        let consumed = remaining_data.len() - reader.into_inner().into_inner().len();
        self.position += consumed;

        Ok(Memo::from_core(core_memo))
    }

    /// Read raw CBOR data as bytes
    #[wasm_bindgen]
    pub fn read_raw(&mut self) -> Result<Vec<u8>, JsError> {
        if self.position >= self.data.len() {
            return Err(JsError::new("End of sequence reached"));
        }

        let remaining_data = &self.data[self.position..];
        let cursor = Cursor::new(remaining_data);
        let mut reader = CoreCborSeqReader::new(cursor);

        // Read as raw CBOR value and serialize back to bytes
        let value: Value = reader.read_block().map_err(|e| match e {
            CoreError::Eof => JsError::new("End of sequence reached"),
            _ => JsError::new(&e.to_string()),
        })?;

        // Update position
        let consumed = remaining_data.len() - reader.into_inner().into_inner().len();
        self.position += consumed;

        // Serialize the value back to CBOR
        let cbor_bytes =
            serde_cbor_core::to_vec(&value).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(cbor_bytes)
    }

    /// Check if there's more data to read
    #[wasm_bindgen]
    pub fn has_more(&self) -> bool {
        self.position < self.data.len()
    }

    /// Reset the reader to the beginning
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.position = 0;
    }

    /// Get the current position in the sequence
    #[wasm_bindgen]
    pub fn position(&self) -> usize {
        self.position
    }

    /// Get the total length of the data
    #[wasm_bindgen]
    pub fn length(&self) -> usize {
        self.data.len()
    }
}

/// WASM wrapper for writing CBOR sequences
#[derive(Debug, Default)]
#[wasm_bindgen]
pub struct CborSeqWriter {
    data: Vec<u8>,
}

#[wasm_bindgen]
impl CborSeqWriter {
    /// Create a new CBOR sequence writer
    #[wasm_bindgen(constructor)]
    pub fn new() -> CborSeqWriter {
        Self { data: Vec::new() }
    }

    /// Write a memo to the sequence
    #[wasm_bindgen]
    pub fn write_memo(&mut self, memo: &Memo) -> Result<(), JsError> {
        let mut writer = CoreCborSeqWriter::new(&mut self.data);
        writer.write_block(memo.as_core())?;
        writer.flush()?;
        Ok(())
    }

    /// Write raw CBOR data to the sequence
    #[wasm_bindgen]
    pub fn write_raw(&mut self, cbor_data: &[u8]) -> Result<(), JsError> {
        // Parse the CBOR data to validate it
        let _value: Value =
            serde_cbor_core::from_slice(cbor_data).map_err(|e| JsError::new(&e.to_string()))?;

        // Write directly to our buffer
        self.data.extend_from_slice(cbor_data);
        Ok(())
    }

    /// Write a JavaScript object as CBOR (must be serializable)
    #[wasm_bindgen]
    pub fn write_object(&mut self, js_value: &JsValue) -> Result<(), JsError> {
        // Convert JsValue to a Rust value that can be serialized
        let value: Value = serde_wasm_bindgen::from_value(js_value.clone())
            .map_err(|e| JsError::new(&e.to_string()))?;

        let mut writer = CoreCborSeqWriter::new(&mut self.data);
        writer.write_block(&value)?;
        writer.flush()?;
        Ok(())
    }

    /// Get the written data as bytes
    #[wasm_bindgen]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.data.clone()
    }

    /// Clear the writer buffer
    #[wasm_bindgen]
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Get the current size of the buffer
    #[wasm_bindgen]
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Check if the buffer is empty
    #[wasm_bindgen]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

// Utility functions for working with CBOR data

/// Parse CBOR data and return it as a JavaScript value
#[wasm_bindgen]
pub fn parse_cbor(data: &[u8]) -> Result<JsValue, JsError> {
    let value: Value =
        serde_cbor_core::from_slice(data).map_err(|e| JsError::new(&e.to_string()))?;
    let js_value =
        serde_wasm_bindgen::to_value(&value).map_err(|e| JsError::new(&e.to_string()))?;
    Ok(js_value)
}

/// Serialize a JavaScript value to CBOR data
#[wasm_bindgen]
pub fn serialize_cbor(js_value: &JsValue) -> Result<Vec<u8>, JsError> {
    let value: Value = serde_wasm_bindgen::from_value(js_value.clone())
        .map_err(|e| JsError::new(&e.to_string()))?;
    let cbor_bytes = serde_cbor_core::to_vec(&value).map_err(|e| JsError::new(&e.to_string()))?;
    Ok(cbor_bytes)
}

/// Validate that data is valid CBOR
#[wasm_bindgen]
pub fn is_valid_cbor(data: &[u8]) -> bool {
    serde_cbor_core::from_slice::<Value>(data).is_ok()
}
