use wasm_bindgen::prelude::*;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Import the `console.log` function from the browser
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Define a macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! console_log {
    ( $( $t:tt )* ) => {
        log(&format!( $( $t )* ))
    }
}

// WASM wrapper modules
pub mod hash;
pub mod ed25519_key_material;
pub mod memo;
pub mod did_key;
pub mod mnemonic;
pub mod cbor_seq;

// Re-export main types for easy use
pub use hash::Hash;
pub use ed25519_key_material::Ed25519KeyMaterial;
pub use memo::Memo;
pub use did_key::DidKey;
pub use mnemonic::Mnemonic;
pub use cbor_seq::{CborSeqReader, CborSeqWriter};

// Utility function to initialize the WASM module
#[wasm_bindgen(start)]
pub fn main() {
    console_log!("SZDT WASM module loaded");
}