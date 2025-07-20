# Rust WebAssembly Dependency Configuration

## User

What do I need to add to my dependencies to enable this library to compile to WASM?

## Agent

I'll help you identify what dependencies you need to compile your Rust library to WebAssembly (WASM). Let me first explore your project structure to understand what you're working with.

**Use Tool: list_directory (toolu_01JHTVqQNLo6iPJVsXG4bvBK)**
```json
{
  "path": "szdt_rust"
}
```

**Tool Results: toolu_01JHTVqQNLo6iPJVsXG4bvBK**

# Folders:
szdt_rust/.claude
szdt_rust/.github
szdt_rust/docs
szdt_rust/fixtures
szdt_rust/research
szdt_rust/src
szdt_rust/target
szdt_rust/website

# Files:
szdt_rust/.gitignore
szdt_rust/CLAUDE.md
szdt_rust/Cargo.lock
szdt_rust/Cargo.toml
szdt_rust/LICENSE-MIT
szdt_rust/README.md
szdt_rust/log.md
szdt_rust/spec.md

## Agent

Now let me examine your `Cargo.toml` to see your current dependencies and project setup:

**Use Tool: read_file (toolu_01ScDK4TxmG9Vw1vNi3wSCVA)**
```json
{
  "path": "szdt_rust/Cargo.toml"
}
```

**Tool Results: toolu_01ScDK4TxmG9Vw1vNi3wSCVA**

[package]
name = "szdt"
version = "0.0.1"
edition = "2024"

[[bin]]
name = "szdt"
path = "src/bin/szdt.rs"

[dependencies]
bip39 = "2.2.0"
blake3 = { version = "1.8.2", features = ["serde"] }
bs58 = "0.5.1"
cbor4ii = { version = "1.0.0", features = ["serde", "serde1", "use_alloc"] }
clap = { version = "4.5.31", features = ["derive"] }
console = "0.16.0"
data-encoding = "2.8.0"
dialoguer = "0.11.0"
dirs = "6.0.0"
ed25519-dalek = { version = "2.2.0", features = [
    "alloc",
    "digest",
    "serde",
    "signature",
] }
mime_guess2 = "2.3.1"
rand = "0.9.1"
rusqlite = "0.37.0"
serde = { version = "1.0.219", features = ["derive"] }
serde_ipld_dagcbor = "=0.6.3"
thiserror = "2.0.12"

[dev-dependencies]
tempfile = "3.19.1"

## Agent

Let me also check if there are any existing WASM-related configurations or build scripts:

**Use Tool: find_path (toolu_014NEEf47w2VDdwkQniBGyNg)**
```json
{
  "glob": "**/*.toml"
}
```

**Tool Results: toolu_014NEEf47w2VDdwkQniBGyNg**

Found 1 total matches.
szdt_rust/Cargo.toml


Debug Output:

```json
{
  "glob": "**/*.toml",
  "paths": [
    "szdt_rust/Cargo.toml"
  ]
}
```

## Agent



**Use Tool: find_path (toolu_01ABuWMbmT1psbrxxDVKWi1j)**
```json
{
  "glob": "**/wasm*"
}
```

**Tool Results: toolu_01ABuWMbmT1psbrxxDVKWi1j**

No matches found
## Agent



**Use Tool: find_path (toolu_01PYmPhDZnEY3TZkBDpXhaG1)**
```json
{
  "glob": "**/*wasm*"
}
```

**Tool Results: toolu_01PYmPhDZnEY3TZkBDpXhaG1**

Found 1 total matches.
szdt_rust/research/2025-01-19-wasm-plan.md


Debug Output:

```json
{
  "glob": "**/*wasm*",
  "paths": [
    "szdt_rust/research/2025-01-19-wasm-plan.md"
  ]
}
```

## Agent

Let me check your WASM plan document to understand your goals:

**Use Tool: read_file (toolu_0154jYxRuKveBfzrKRid7ycu)**
```json
{
  "path": "szdt_rust/research/2025-01-19-wasm-plan.md"
}
```

**Tool Results: toolu_0154jYxRuKveBfzrKRid7ycu**

# WASM Integration Plan for SZDT Library

*Date: 2025-01-19*

## Overview

This document outlines the plan for exposing core SZDT Rust library functionality as a WebAssembly (WASM) library for JavaScript/TypeScript environments. The focus is on cryptographic operations, data structures, and serialization while excluding CLI-specific functionality like archiving.

## Core Modules for WASM Export

### 1. Key Material Operations (`ed25519_key_material.rs`)
- `Ed25519KeyMaterial::generate()` - Generate new key material
- `Ed25519KeyMaterial::from_seed(seed)` - Create from seed
- `Ed25519KeyMaterial::from_mnemonic(mnemonic)` - Create from mnemonic
- `Ed25519KeyMaterial::did()` - Get DID key
- `Ed25519KeyMaterial::sign(data)` - Sign data
- `Ed25519KeyMaterial::verify(data, signature)` - Verify signatures

### 2. Hash Operations (`hash.rs`)
- `Hash::new(data)` - Blake3 hashing
- `Hash::from_reader()` - Streaming hash computation
- `Hash::from_bytes()` - Create from byte array
- `Hash::as_bytes()` - Get hash bytes
- `Hash::to_string()` - Base32 representation

### 3. DID Key Management (`did.rs`)
- `DidKey::new(pubkey_bytes)` - Create DID from public key
- `DidKey::parse(did_key_url)` - Parse DID:key URLs
- `DidKey::to_string()` - Serialize DID to string

### 4. Memo Operations (`memo.rs`)
- `Memo::new(body_hash)` - Create new memo
- `Memo::for_body(content)` - Create memo for content
- `Memo::sign(key_material)` - Sign memo
- `Memo::verify()` - Verify memo signature
- `Memo::validate(timestamp)` - Full validation including time checks

### 5. Serialization (`cbor_seq.rs`, `bytes.rs`)
- `CborSeqWriter` - CBOR sequence writing
- `CborSeqReader` - CBOR sequence reading
- Bytes wrapper for binary data

## WASM Architecture

### 1. Cargo.toml Configuration

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
js-sys = "0.3"
web-sys = "0.3"
serde-wasm-bindgen = "0.6"
```

### 2. WASM API Structure

The approach is to expose methods directly on the original Rust structs using `#[wasm_bindgen]` implementations, avoiding wrapper types for clean JavaScript APIs.

### 3. Key Features to Expose

#### Ed25519KeyMaterial API (Direct exposure)
```rust
#[wasm_bindgen]
impl Ed25519KeyMaterial {
    #[wasm_bindgen]
    pub fn generate() -> Ed25519KeyMaterial
    
    #[wasm_bindgen]
    pub fn from_seed(seed: &[u8]) -> Result<Ed25519KeyMaterial, JsError>
    
    #[wasm_bindgen]
    pub fn from_mnemonic(mnemonic: &str) -> Result<Ed25519KeyMaterial, JsError>
    
    #[wasm_bindgen]
    pub fn did_string(&self) -> String
    
    #[wasm_bindgen]
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, JsError>
    
    #[wasm_bindgen]
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, JsError>
}
```

#### Hash API (Direct exposure)
```rust
#[wasm_bindgen]
impl Hash {
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8]) -> Hash
    
    #[wasm_bindgen]
    pub fn from_bytes(bytes: &[u8]) -> Result<Hash, JsError>
    
    #[wasm_bindgen]
    pub fn as_bytes(&self) -> Vec<u8>
    
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String
}
```

#### Memo API (Direct exposure)
```rust
#[wasm_bindgen]
impl Memo {
    #[wasm_bindgen(constructor)]
    pub fn new(body_hash: &Hash) -> Memo
    
    #[wasm_bindgen]
    pub fn for_body(content: &[u8]) -> Result<Memo, JsError>
    
    #[wasm_bindgen]
    pub fn sign(&mut self, key_material: &Ed25519KeyMaterial) -> Result<(), JsError>
    
    #[wasm_bindgen]
    pub fn verify(&self) -> Result<bool, JsError>
    
    #[wasm_bindgen]
    pub fn validate(&self, timestamp: Option<u64>) -> Result<bool, JsError>
    
    #[wasm_bindgen]
    pub fn to_cbor(&self) -> Result<Vec<u8>, JsError>
    
    #[wasm_bindgen]
    pub fn from_cbor(data: &[u8]) -> Result<Memo, JsError>
    
    #[wasm_bindgen]
    pub fn checksum(&self, body_hash: &Hash) -> Result<bool, JsError>
}
```

### 4. TypeScript Definitions

Generate comprehensive TypeScript definitions using `wasm-bindgen` for:
- Ed25519KeyMaterial management (directly on Ed25519KeyMaterial class)
- Hash operations (directly on Hash class)
- Memo operations (directly on Memo class)
- DID key operations
- CBOR serialization/deserialization

### 5. Build Configuration

- Use `wasm-pack` for building and packaging
- Target both Node.js and browser environments
- Optimize for size with `wee_alloc` and `lto = true`
- Consider splitting into multiple WASM modules if size becomes an issue

### 6. JavaScript/TypeScript Usage Example

```typescript
import { Hash, Memo, Ed25519KeyMaterial } from './szdt_wasm';

// Direct hash usage
const data = new Uint8Array([1, 2, 3, 4]);
const hash = new Hash(data);
console.log(hash.toString()); // Base32 representation
const hashBytes = hash.as_bytes();

// Direct key material usage
const keyMaterial = Ed25519KeyMaterial.generate();
console.log(keyMaterial.did_string());

// Direct memo usage
const memo = new Memo(hash);
memo.sign(keyMaterial);
const isValid = memo.verify();

// Or create memo from content
const memo2 = Memo.for_body(data);
memo2.sign(keyMaterial);
const cborData = memo2.to_cbor();
```

## Implementation Notes

### Excluded Functionality
- Archive creation and reading operations (CLI-specific)
- File system operations
- Database operations

### Design Principles
- **Direct exposure**: Expose methods directly on original structs rather than creating wrapper types
- **Clean APIs**: Provide ergonomic JavaScript/TypeScript interfaces
- **Memory safety**: Ensure proper memory management in WASM context
- **Error handling**: Convert Rust Result types to JavaScript exceptions appropriately
- **Type safety**: Generate comprehensive TypeScript definitions

### Future Considerations
- Streaming interfaces using Web Streams API for large data processing
- Async/await patterns for computationally heavy operations
- Progressive loading strategies if bundle size becomes an issue
- Integration with existing JavaScript cryptography libraries where beneficial

This architecture provides the core SZDT functionality in a clean, type-safe manner suitable for web and Node.js applications while maintaining the security and integrity features of the original Rust implementation.
## Agent

Now let me also check your current source code to understand what you're working with:

**Use Tool: list_directory (toolu_01YA18gTEYSoa9gFEqit2vtr)**
```json
{
  "path": "szdt_rust/src"
}
```

**Tool Results: toolu_01YA18gTEYSoa9gFEqit2vtr**

# Folders:
szdt_rust/src/bin
szdt_rust/src/db

# Files:
szdt_rust/src/base58btc.rs
szdt_rust/src/bytes.rs
szdt_rust/src/cbor_seq.rs
szdt_rust/src/config.rs
szdt_rust/src/contact.rs
szdt_rust/src/content_type.rs
szdt_rust/src/db.rs
szdt_rust/src/did.rs
szdt_rust/src/ed25519.rs
szdt_rust/src/ed25519_key_material.rs
szdt_rust/src/error.rs
szdt_rust/src/file.rs
szdt_rust/src/hash.rs
szdt_rust/src/hashseq.rs
szdt_rust/src/key_storage.rs
szdt_rust/src/lib.rs
szdt_rust/src/link.rs
szdt_rust/src/manifest.rs
szdt_rust/src/memo.rs
szdt_rust/src/mnemonic.rs
szdt_rust/src/nickname.rs
szdt_rust/src/szdt.rs
szdt_rust/src/text.rs
szdt_rust/src/time.rs
szdt_rust/src/value.rs

## Agent

Perfect! Based on your project structure and your excellent WASM plan document, I can see exactly what you need to add to your dependencies to enable WASM compilation. Here's what you need to add to your `Cargo.toml`:

## Required Dependencies for WASM Compilation

You'll need to add the following dependencies to your `[dependencies]` section:

```szdt_rust/Cargo.toml#L1-50
[dependencies]
# ... your existing dependencies ...

# WASM-specific dependencies
wasm-bindgen = "0.2"
js-sys = "0.3"
web-sys = "0.3"
serde-wasm-bindgen = "0.6"
wee_alloc = "0.4"  # Optional: for smaller WASM binary size
```

## Cargo.toml Configuration Changes

You'll also need to add library configuration to support both your current binary and the new WASM library:

```szdt_rust/Cargo.toml#L51-55
[lib]
name = "szdt"
crate-type = ["cdylib", "rlib"]  # cdylib for WASM, rlib for regular Rust usage
```

## Dependency Compatibility Check

Looking at your current dependencies, most should work fine with WASM, but you may need to make some adjustments:

### ⚠️ Potential Issues:
1. **`rusqlite`** - This won't work in WASM as it requires native SQLite. You'll need to conditionally exclude this for WASM builds.
2. **`dirs`** - File system access won't work in WASM browser context.
3. **`console`** and **`dialoguer`** - These are CLI-specific and won't work in WASM.

### Conditional Compilation

You can handle platform-specific dependencies like this:

```szdt_rust/Cargo.toml#L56-70
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
rusqlite = "0.37.0"
dirs = "6.0.0"
console = "0.16.0"
dialoguer = "0.11.0"

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
js-sys = "0.3" 
web-sys = "0.3"
serde-wasm-bindgen = "0.6"
wee_alloc = "0.4"
```

## Complete Cargo.toml Update

Here's how your updated `Cargo.toml` should look:

```szdt_rust/Cargo.toml#L1-80
[package]
name = "szdt"
version = "0.0.1"
edition = "2024"

[lib]
name = "szdt"
crate-type = ["cdylib", "rlib"]

[[bin]]
name = "szdt"
path = "src/bin/szdt.rs"

[dependencies]
bip39 = "2.2.0"
blake3 = { version = "1.8.2", features = ["serde"] }
bs58 = "0.5.1"
cbor4ii = { version = "1.0.0", features = ["serde", "serde1", "use_alloc"] }
clap = { version = "4.5.31", features = ["derive"] }
data-encoding = "2.8.0"
ed25519-dalek = { version = "2.2.0", features = [
    "alloc",
    "digest",
    "serde",
    "signature",
] }
mime_guess2 = "2.3.1"
rand = "0.9.1"
serde = { version = "1.0.219", features = ["derive"] }
serde_ipld_dagcbor = "=0.6.3"
thiserror = "2.0.12"

# Platform-specific dependencies
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
rusqlite = "0.37.0"
dirs = "6.0.0"
console = "0.16.0"
dialoguer = "0.11.0"

# WASM-specific dependencies
[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
js-sys = "0.3"
web-sys = "0.3"
serde-wasm-bindgen = "0.6"
wee_alloc = "0.4"

[dev-dependencies]
tempfile = "3.19.1"
```

## Additional Setup Steps

1. **Install wasm-pack** (if you haven't already):
   ```bash
   cargo install wasm-pack
   ```

2. **Add build optimization for WASM** (optional but recommended):
   ```szdt_rust/Cargo.toml#L81-90
   [profile.release]
   lto = true
   opt-level = "s"  # Optimize for size
   ```

3. **You'll also need to modify your source code** to add conditional compilation attributes for WASM-incompatible modules and add `#[wasm_bindgen]` annotations as outlined in your plan.

This setup will allow you to compile your library to WASM while maintaining compatibility with your existing CLI binary. The conditional compilation ensures that WASM-incompatible dependencies are only included for non-WASM targets.

## User

How do I conditionally exclude modules/functions/methods?

## Agent

Great question! Rust provides several ways to conditionally exclude code based on the target architecture or features. Here are the main approaches you'll need for your WASM compilation:

## 1. Module-Level Conditional Compilation

Use `#[cfg()]` attributes to exclude entire modules:

```szdt_rust/src/lib.rs#L1-15
// Include these modules only for non-WASM targets
#[cfg(not(target_arch = "wasm32"))]
pub mod db;
#[cfg(not(target_arch = "wasm32"))]
pub mod key_storage;
#[cfg(not(target_arch = "wasm32"))]
pub mod config;

// Include these modules for all targets
pub mod hash;
pub mod memo;
pub mod ed25519_key_material;
pub mod did;
pub mod bytes;
pub mod cbor_seq;
```

## 2. Function-Level Conditional Compilation

Exclude specific functions within a module:

```szdt_rust/src/file.rs#L1-20
use std::path::Path;

pub struct FileOps;

impl FileOps {
    // This function only available on non-WASM targets
    #[cfg(not(target_arch = "wasm32"))]
    pub fn read_from_disk(path: &Path) -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(path)
    }
    
    // This function available on all targets
    pub fn validate_content(data: &[u8]) -> bool {
        !data.is_empty()
    }
    
    // WASM-specific alternative
    #[cfg(target_arch = "wasm32")]
    pub fn read_from_browser_file(file_data: &[u8]) -> Result<Vec<u8>, String> {
        Ok(file_data.to_vec())
    }
}
```

## 3. Struct Fields and Implementations

Conditionally include fields in structs:

```szdt_rust/src/memo.rs#L1-25
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memo {
    pub body_hash: crate::hash::Hash,
    pub signature: Option<Vec<u8>>,
    pub timestamp: u64,
    
    // Only include file path for non-WASM builds
    #[cfg(not(target_arch = "wasm32"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<std::path::PathBuf>,
}

impl Memo {
    // Available on all targets
    pub fn new(body_hash: crate::hash::Hash) -> Self {
        Self {
            body_hash,
            signature: None,
            timestamp: crate::time::current_timestamp(),
            #[cfg(not(target_arch = "wasm32"))]
            file_path: None,
        }
    }
    
    // File system operations only for non-WASM
    #[cfg(not(target_arch = "wasm32"))]
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), std::io::Error> {
        // File saving logic here
        Ok(())
    }
}
```

## 4. Import/Use Statements

Conditionally import dependencies:

```szdt_rust/src/key_storage.rs#L1-15
// Only import rusqlite for non-WASM targets
#[cfg(not(target_arch = "wasm32"))]
use rusqlite::{Connection, Result as SqlResult};

#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

// WASM-specific imports
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use js_sys::*;
```

## 5. Alternative Implementations

Provide different implementations for different targets:

```szdt_rust/src/hash.rs#L1-40
use blake3::Hasher;

pub struct Hash {
    bytes: [u8; 32],
}

impl Hash {
    // Common implementation for all targets
    pub fn new(data: &[u8]) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(data);
        Self {
            bytes: hasher.finalize().into(),
        }
    }
    
    // File-based hashing for non-WASM
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, std::io::Error> {
        let data = std::fs::read(path)?;
        Ok(Self::new(&data))
    }
    
    // Browser File API for WASM
    #[cfg(target_arch = "wasm32")]
    pub fn from_browser_file(file_data: &[u8]) -> Self {
        Self::new(file_data)
    }
    
    // Different string representations
    #[cfg(not(target_arch = "wasm32"))]
    pub fn to_base58(&self) -> String {
        bs58::encode(&self.bytes).into_string()
    }
    
    #[cfg(target_arch = "wasm32")]
    pub fn to_base64(&self) -> String {
        use data_encoding::BASE64;
        BASE64.encode(&self.bytes)
    }
}
```

## 6. Feature-Based Conditional Compilation

You can also use custom features instead of just target architecture:

```szdt_rust/Cargo.toml#L1-10
[features]
default = ["cli"]
cli = ["rusqlite", "dirs", "console", "dialoguer"]
wasm = ["wasm-bindgen", "js-sys", "web-sys"]
```

Then use in code:

```szdt_rust/src/lib.rs#L1-10
#[cfg(feature = "cli")]
pub mod db;
#[cfg(feature = "cli")]
pub mod key_storage;

#[cfg(feature = "wasm")]
pub mod wasm_bindings;
```

## 7. Complex Conditions

You can combine conditions:

```szdt_rust/src/example.rs#L1-15
// Only for non-WASM Unix systems
#[cfg(all(not(target_arch = "wasm32"), target_family = "unix"))]
pub fn unix_specific_function() {}

// For WASM or Windows
#[cfg(any(target_arch = "wasm32", target_os = "windows"))]
pub fn cross_platform_alternative() {}

// Not for WASM, and either debug or test mode
#[cfg(all(not(target_arch = "wasm32"), any(debug_assertions, test)))]
pub fn debug_helper() {}
```

## 8. Conditional Compilation in Your Project

Based on your WASM plan, here's how you might structure your main lib.rs:

```szdt_rust/src/lib.rs#L1-30
// Core modules - available on all targets
pub mod hash;
pub mod memo;
pub mod ed25519_key_material;
pub mod did;
pub mod bytes;
pub mod cbor_seq;
pub mod error;
pub mod time;

// CLI-only modules
#[cfg(not(target_arch = "wasm32"))]
pub mod db;
#[cfg(not(target_arch = "wasm32"))]
pub mod key_storage;
#[cfg(not(target_arch = "wasm32"))]
pub mod config;
#[cfg(not(target_arch = "wasm32"))]
pub mod file;

// WASM-specific modules
#[cfg(target_arch = "wasm32")]
pub mod wasm_bindings;

// Re-export main types for easier usage
pub use hash::Hash;
pub use memo::Memo;
pub use ed25519_key_material::Ed25519KeyMaterial;
pub use did::DidKey;
```

## Common Patterns for Your Use Case

- Use `#[cfg(not(target_arch = "wasm32"))]` for file system, database, and CLI operations
- Use `#[cfg(target_arch = "wasm32")]` for WASM-specific bindings and browser APIs
- Keep cryptographic and core data structure code unconditional
- Use conditional compilation for imports of platform-specific crates

This approach allows you to maintain a single codebase that compiles to both a native CLI tool and a WASM library!

## User

How do I mark a struct/property/method so that it will be exposed to JS/Typescript with wasm-bindgen?

## Agent

Great question! `wasm-bindgen` provides several attributes to expose Rust code to JavaScript/TypeScript. Here's how to mark different elements:

## 1. Basic Struct Exposure

Use `#[wasm_bindgen]` on the struct itself:

```szdt_rust/src/hash.rs#L1-15
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Hash {
    bytes: [u8; 32],  // Private fields are not exposed
}

// You can also expose public fields directly
#[wasm_bindgen]
pub struct SimpleStruct {
    #[wasm_bindgen(skip)]  // Skip this field from JS exposure
    private_data: Vec<u8>,
    
    pub timestamp: u64,  // This will be exposed to JS
}
```

## 2. Method Exposure

Use `#[wasm_bindgen]` on impl blocks and individual methods:

```szdt_rust/src/hash.rs#L16-40
#[wasm_bindgen]
impl Hash {
    // Constructor - use #[wasm_bindgen(constructor)] 
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8]) -> Hash {
        let mut hasher = blake3::Hasher::new();
        hasher.update(data);
        Self {
            bytes: hasher.finalize().into(),
        }
    }
    
    // Regular method
    #[wasm_bindgen]
    pub fn as_bytes(&self) -> Vec<u8> {
        self.bytes.to_vec()
    }
    
    // Method with custom JS name
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        data_encoding::BASE32_NOPAD.encode(&self.bytes)
    }
    
    // Static method
    #[wasm_bindgen]
    pub fn from_bytes(bytes: &[u8]) -> Result<Hash, JsError> {
        if bytes.len() != 32 {
            return Err(JsError::new("Hash must be exactly 32 bytes"));
        }
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(bytes);
        Ok(Self { bytes: hash_bytes })
    }
}
```

## 3. Error Handling

Use `JsError` for errors that will be thrown in JavaScript:

```szdt_rust/src/ed25519_key_material.rs#L1-30
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Ed25519KeyMaterial {
    // Private implementation details
    signing_key: ed25519_dalek::SigningKey,
}

#[wasm_bindgen]
impl Ed25519KeyMaterial {
    #[wasm_bindgen]
    pub fn generate() -> Ed25519KeyMaterial {
        let mut csprng = rand::rngs::OsRng;
        let signing_key = ed25519_dalek::SigningKey::generate(&mut csprng);
        Self { signing_key }
    }
    
    #[wasm_bindgen]
    pub fn from_seed(seed: &[u8]) -> Result<Ed25519KeyMaterial, JsError> {
        if seed.len() != 32 {
            return Err(JsError::new("Seed must be exactly 32 bytes"));
        }
        
        let signing_key = ed25519_dalek::SigningKey::from_bytes(
            seed.try_into().map_err(|_| JsError::new("Invalid seed"))?
        );
        
        Ok(Self { signing_key })
    }
    
    #[wasm_bindgen]
    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        use ed25519_dalek::Signer;
        self.signing_key.sign(data).to_bytes().to_vec()
    }
}
```

## 4. Getter and Setter Methods

For properties, use `#[wasm_bindgen(getter)]` and `#[wasm_bindgen(setter)]`:

```szdt_rust/src/memo.rs#L1-35
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Memo {
    body_hash: crate::hash::Hash,
    signature: Option<Vec<u8>>,
    timestamp: u64,
}

#[wasm_bindgen]
impl Memo {
    #[wasm_bindgen(constructor)]
    pub fn new(body_hash: &crate::hash::Hash) -> Memo {
        Self {
            body_hash: body_hash.clone(),
            signature: None,
            timestamp: js_sys::Date::now() as u64,
        }
    }
    
    // Getter for timestamp
    #[wasm_bindgen(getter)]
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
    
    // Setter for timestamp
    #[wasm_bindgen(setter)]
    pub fn set_timestamp(&mut self, timestamp: u64) {
        self.timestamp = timestamp;
    }
    
    // Method that returns another WASM-bound struct
    #[wasm_bindgen]
    pub fn body_hash(&self) -> crate::hash::Hash {
        self.body_hash.clone()
    }
}
```

## 5. Free Functions

Expose standalone functions:

```szdt_rust/src/lib.rs#L1-20
use wasm_bindgen::prelude::*;

// Free function exposed to JS
#[wasm_bindgen]
pub fn create_hash_from_string(input: &str) -> crate::hash::Hash {
    crate::hash::Hash::new(input.as_bytes())
}

// Function with custom JS name
#[wasm_bindgen(js_name = generateMnemonic)]
pub fn generate_mnemonic_phrase() -> String {
    // Implementation here
    "example mnemonic phrase".to_string()
}

// Function that works with JS values
#[wasm_bindgen]
pub fn process_js_array(arr: &js_sys::Array) -> Result<Vec<u8>, JsError> {
    // Convert JS array to Rust Vec
    let mut result = Vec::new();
    for item in arr.iter() {
        let num = item.as_f64().ok_or_else(|| JsError::new("Expected number"))?;
        result.push(num as u8);
    }
    Ok(result)
}
```

## 6. Advanced Attribute Options

```szdt_rust/src/advanced.rs#L1-25
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct AdvancedExample {
    data: Vec<u8>,
}

#[wasm_bindgen]
impl AdvancedExample {
    // Skip exposing this method to JS
    #[wasm_bindgen(skip)]
    pub fn internal_method(&self) -> bool {
        true
    }
    
    // Method with custom JS name and TypeScript type
    #[wasm_bindgen(js_name = processData, typescript_type = "processData(data: Uint8Array): Promise<Uint8Array>")]
    pub fn process_data(&self, data: &[u8]) -> Vec<u8> {
        data.to_vec()
    }
    
    // Method that takes ownership (consumes self)
    #[wasm_bindgen]
    pub fn consume(self) -> Vec<u8> {
        self.data
    }
}
```

## 7. Working with JavaScript Types

```szdt_rust/src/js_interop.rs#L1-30
use wasm_bindgen::prelude::*;
use js_sys::*;
use web_sys::*;

#[wasm_bindgen]
pub struct JsInterop;

#[wasm_bindgen]
impl JsInterop {
    // Working with JS objects
    #[wasm_bindgen]
    pub fn process_object(obj: &Object) -> Result<String, JsError> {
        let json_string = JSON::stringify(obj)
            .map_err(|_| JsError::new("Failed to stringify object"))?;
        Ok(json_string.as_string().unwrap_or_default())
    }
    
    // Working with Promises (async)
    #[wasm_bindgen]
    pub async fn async_operation() -> Result<String, JsError> {
        // Async operations here
        Ok("completed".to_string())
    }
    
    // Working with typed arrays
    #[wasm_bindgen]
    pub fn process_uint8_array(data: &Uint8Array) -> Vec<u8> {
        data.to_vec()
    }
}
```

## 8. Example: Your Hash Struct Implementation

Based on your project, here's how your Hash struct might look:

```szdt_rust/src/hash.rs#L1-50
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Hash {
    bytes: [u8; 32],
}

#[wasm_bindgen]
impl Hash {
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8]) -> Hash {
        let mut hasher = blake3::Hasher::new();
        hasher.update(data);
        Self {
            bytes: hasher.finalize().into(),
        }
    }
    
    #[wasm_bindgen]
    pub fn from_bytes(bytes: &[u8]) -> Result<Hash, JsError> {
        if bytes.len() != 32 {
            return Err(JsError::new("Hash must be exactly 32 bytes"));
        }
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(bytes);
        Ok(Self { bytes: hash_bytes })
    }
    
    #[wasm_bindgen]
    pub fn as_bytes(&self) -> Vec<u8> {
        self.bytes.to_vec()
    }
    
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        data_encoding::BASE32_NOPAD.encode(&self.bytes)
    }
    
    #[wasm_bindgen]
    pub fn equals(&self, other: &Hash) -> bool {
        self.bytes == other.bytes
    }
}
```

## Key Points:

1. **`#[wasm_bindgen]`** on structs and impl blocks exposes them to JS
2. **`#[wasm_bindgen(constructor)]`** creates a JS constructor
3. **`#[wasm_bindgen(getter/setter)]`** creates property accessors
4. **`#[wasm_bindgen(js_name = "...")]`** customizes the JavaScript name
5. **`JsError`** for throwing JavaScript exceptions
6. **`#[wasm_bindgen(skip)]`** excludes methods from JS exposure
7. Private struct fields are never exposed; use methods for access

This will generate TypeScript definitions automatically when you build with `wasm-pack`!

## User

What is JsError?

## Agent



**Use Tool: thinking (toolu_01GU8oEJDQ9peRt3X6iUYnxx)**
```json
{
  "content": "The user is asking about JsError, which I mentioned in my previous response about wasm-bindgen. I should explain what JsError is, where it comes from, how to use it, and provide examples. Let me give a comprehensive explanation of JsError in the context of wasm-bindgen and Rust-to-JavaScript interop."
}
```

**Tool Results: toolu_01GU8oEJDQ9peRt3X6iUYnxx**

Finished thinking.
## Agent

`JsError` is a type provided by `wasm-bindgen` that represents JavaScript errors in Rust code. It's the bridge between Rust's error handling (using `Result<T, E>`) and JavaScript's exception throwing.

## What is JsError?

`JsError` is a wrapper around JavaScript's native `Error` object that can be:
1. Created in Rust code
2. Thrown as a JavaScript exception when returned from WASM functions
3. Caught and handled in JavaScript/TypeScript code

## Import and Basic Usage

```szdt_rust/src/example.rs#L1-15
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn might_fail(value: i32) -> Result<i32, JsError> {
    if value < 0 {
        // Create a new JsError with a message
        Err(JsError::new("Value cannot be negative"))
    } else {
        Ok(value * 2)
    }
}
```

## Creating JsErrors

### 1. From a String Message
```szdt_rust/src/errors.rs#L1-10
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn create_simple_error() -> Result<(), JsError> {
    Err(JsError::new("Something went wrong"))
}

// With dynamic message
#[wasm_bindgen]
pub fn validate_age(age: u32) -> Result<String, JsError> {
    if age > 150 {
        Err(JsError::new(&format!("Age {} is unrealistic", age)))
    } else {
        Ok(format!("Age {} is valid", age))
    }
}
```

### 2. From Rust Errors
```szdt_rust/src/conversions.rs#L1-25
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse_number(input: &str) -> Result<i32, JsError> {
    // Convert Rust's ParseIntError to JsError
    input.parse::<i32>()
        .map_err(|e| JsError::new(&format!("Failed to parse '{}': {}", input, e)))
}

// Using the ? operator with custom conversion
#[wasm_bindgen]  
pub fn read_and_parse(data: &str) -> Result<Vec<i32>, JsError> {
    let mut numbers = Vec::new();
    
    for line in data.lines() {
        let num = line.parse::<i32>()
            .map_err(|_| JsError::new(&format!("Invalid number: {}", line)))?;
        numbers.push(num);
    }
    
    Ok(numbers)
}
```

### 3. From JavaScript Values
```szdt_rust/src/js_errors.rs#L1-15
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn handle_js_error(js_value: JsValue) -> Result<String, JsError> {
    // Convert a JsValue to JsError
    if js_value.is_null() {
        Err(JsError::from(js_value))
    } else {
        Ok("Valid value".to_string())
    }
}
```

## Converting Rust Errors to JsError

### Manual Conversion
```szdt_rust/src/error_conversion.rs#L1-30
use wasm_bindgen::prelude::*;
use std::io;

#[wasm_bindgen]
pub fn file_operation() -> Result<String, JsError> {
    // Manually convert std::io::Error
    std::fs::read_to_string("config.txt")
        .map_err(|e| JsError::new(&format!("File error: {}", e)))
}

// Using a helper function
fn io_error_to_js(e: io::Error) -> JsError {
    JsError::new(&format!("IO Error: {}", e))
}

#[wasm_bindgen]
pub fn another_file_operation() -> Result<Vec<u8>, JsError> {
    std::fs::read("data.bin")
        .map_err(io_error_to_js)
}
```

### Using From/Into Traits
```szdt_rust/src/custom_error.rs#L1-35
use wasm_bindgen::prelude::*;
use thiserror::Error;

// Define your own error type
#[derive(Error, Debug)]
pub enum MyError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Parse error")]
    Parse,
}

// Implement conversion to JsError
impl From<MyError> for JsError {
    fn from(err: MyError) -> JsError {
        JsError::new(&err.to_string())
    }
}

#[wasm_bindgen]
pub fn complex_operation(input: &str) -> Result<String, JsError> {
    if input.is_empty() {
        return Err(MyError::InvalidInput("empty string".to_string()).into());
    }
    
    if input.len() > 100 {
        return Err(MyError::InvalidInput("too long".to_string()).into());
    }
    
    Ok(input.to_uppercase())
}
```

## JavaScript/TypeScript Usage

When you use these functions in JavaScript, `JsError` becomes a regular JavaScript Error:

```/dev/null/example.js#L1-20
import { might_fail, validate_age, parse_number } from './pkg/your_wasm_module.js';

try {
    const result = might_fail(-5);
    console.log(result);
} catch (error) {
    console.error('Caught error:', error.message); // "Value cannot be negative"
    console.error('Error type:', error instanceof Error); // true
}

// Async error handling
async function handleErrors() {
    try {
        await validate_age(200);
    } catch (err) {
        console.log('Validation failed:', err.message); // "Age 200 is unrealistic"
    }
    
    try {
        const num = parse_number("not-a-number");
    } catch (err) {
        console.log('Parse error:', err.message); // Contains the parse error details
    }
}
```

## TypeScript Types

With proper TypeScript definitions, errors are properly typed:

```/dev/null/example.ts#L1-15
import { might_fail, validate_age } from './pkg/your_wasm_module';

// TypeScript knows these can throw
async function example() {
    try {
        const result = might_fail(10); // Returns number
        const ageResult = validate_age(25); // Returns string
    } catch (error: unknown) {
        if (error instanceof Error) {
            console.log('Error message:', error.message);
        }
    }
}
```

## Best Practices for Your Project

Based on your SZDT project, here's how you might use JsError:

```szdt_rust/src/ed25519_key_material.rs#L1-40
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl Ed25519KeyMaterial {
    #[wasm_bindgen]
    pub fn from_mnemonic(mnemonic: &str) -> Result<Ed25519KeyMaterial, JsError> {
        // Validate mnemonic format
        if mnemonic.split_whitespace().count() != 12 {
            return Err(JsError::new("Mnemonic must contain exactly 12 words"));
        }
        
        // Try to parse mnemonic
        let seed = crate::mnemonic::mnemonic_to_seed(mnemonic)
            .map_err(|e| JsError::new(&format!("Invalid mnemonic: {}", e)))?;
            
        Self::from_seed(&seed)
    }
    
    #[wasm_bindgen]
    pub fn from_seed(seed: &[u8]) -> Result<Ed25519KeyMaterial, JsError> {
        if seed.len() != 32 {
            return Err(JsError::new(&format!(
                "Seed must be exactly 32 bytes, got {}", 
                seed.len()
            )));
        }
        
        let signing_key = ed25519_dalek::SigningKey::from_bytes(
            seed.try_into().map_err(|_| JsError::new("Failed to create signing key"))?
        );
        
        Ok(Self { signing_key })
    }
    
    #[wasm_bindgen]
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, JsError> {
        if data.is_empty() {
            return Err(JsError::new("Cannot sign empty data"));
        }
        
        use ed25519_dalek::Signer;
        Ok(self.signing_key.sign(data).to_bytes().to_vec())
    }
}
```

## Key Points:

1. **`JsError::new(message)`** - Creates a new JavaScript Error with a message
2. **`Result<T, JsError>`** - Return type that becomes a throwing function in JS
3. **Automatic conversion** - When returned from WASM functions, becomes JavaScript exceptions
4. **Error chaining** - Use `.map_err()` to convert Rust errors to JsError
5. **Rich error messages** - Include context and details in error messages
6. **Type safety** - TypeScript will properly type functions that can throw

`JsError` is essential for proper error handling when exposing Rust functionality to JavaScript!

## User

Is it typical to wrap Rust types in a newtype for WASM, so that we can have a shared core and expose only certain methods and return JsErrors in Result? 

## Agent



**Use Tool: thinking (toolu_0193FcfKXP1uXru8iuhRKeSu)**
```json
{
  "content": "This is a great question about WASM architecture patterns. The user is asking about whether it's common to create wrapper types (newtypes) around core Rust types when exposing them to WASM, rather than directly exposing the core types with wasm-bindgen attributes.\n\nLet me think about the different approaches:\n\n1. **Direct exposure approach**: Put #[wasm_bindgen] directly on your core types\n   - Pros: Simple, no duplication\n   - Cons: Pollutes core types with WASM-specific code, harder to maintain different APIs\n\n2. **Wrapper/newtype approach**: Create WASM-specific wrapper types\n   - Pros: Clean separation, different APIs for different targets, better error handling\n   - Cons: More boilerplate, potential duplication\n\n3. **Hybrid approach**: Use conditional compilation on the same types\n\nThe user seems to be thinking about the wrapper approach, which is actually quite common and has several benefits. Let me explain the different patterns and their trade-offs."
}
```

**Tool Results: toolu_0193FcfKXP1uXru8iuhRKeSu**

Finished thinking.
## Agent

Yes, it's quite common and often considered a best practice to use wrapper types (newtypes) for WASM exposure! This pattern provides several important benefits. Let me show you the different approaches and why wrappers are often preferred.

## Approach 1: Direct Exposure (Simple but Limited)

```szdt_rust/src/hash_direct.rs#L1-25
// Direct approach - putting wasm-bindgen on core types
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Debug, Clone)]
pub struct Hash {
    bytes: [u8; 32],
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl Hash {
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8]) -> Hash {
        // Core implementation
        let mut hasher = blake3::Hasher::new();
        hasher.update(data);
        Self {
            bytes: hasher.finalize().into(),
        }
    }
}

// But this pollutes your core type with WASM-specific concerns
```

## Approach 2: Wrapper Types (Recommended Pattern)

```szdt_rust/src/hash_core.rs#L1-30
// Core implementation - clean, no WASM dependencies
#[derive(Debug, Clone, PartialEq)]
pub struct Hash {
    bytes: [u8; 32],
}

impl Hash {
    pub fn new(data: &[u8]) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(data);
        Self {
            bytes: hasher.finalize().into(),
        }
    }
    
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self { bytes }
    }
    
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }
    
    pub fn to_base32(&self) -> String {
        data_encoding::BASE32_NOPAD.encode(&self.bytes)
    }
}
```

```szdt_rust/src/wasm/hash_wrapper.rs#L1-55
// WASM wrapper - only compiled for WASM target
use wasm_bindgen::prelude::*;
use crate::hash_core::Hash as CoreHash;

#[wasm_bindgen]
pub struct Hash {
    inner: CoreHash,
}

#[wasm_bindgen]
impl Hash {
    #[wasm_bindgen(constructor)]
    pub fn new(data: &[u8]) -> Hash {
        Self {
            inner: CoreHash::new(data),
        }
    }
    
    #[wasm_bindgen]
    pub fn from_bytes(bytes: &[u8]) -> Result<Hash, JsError> {
        if bytes.len() != 32 {
            return Err(JsError::new(&format!(
                "Hash must be exactly 32 bytes, got {}", 
                bytes.len()
            )));
        }
        
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(bytes);
        
        Ok(Self {
            inner: CoreHash::from_bytes(hash_bytes),
        })
    }
    
    #[wasm_bindgen]
    pub fn as_bytes(&self) -> Vec<u8> {
        self.inner.as_bytes().to_vec()
    }
    
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        self.inner.to_base32()
    }
    
    #[wasm_bindgen]
    pub fn equals(&self, other: &Hash) -> bool {
        self.inner == other.inner
    }
}

// Conversion methods for internal use
impl Hash {
    pub fn from_core(core: CoreHash) -> Self {
        Self { inner: core }
    }
    
    pub fn into_core(self) -> CoreHash {
        self.inner
    }
}
```

## Approach 3: More Complex Example with Error Handling

```szdt_rust/src/ed25519_core.rs#L1-40
// Core Ed25519 implementation
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};

#[derive(Debug)]
pub enum Ed25519Error {
    InvalidSeed,
    InvalidSignature,
    SigningFailed,
    InvalidMnemonic(String),
}

#[derive(Debug, Clone)]
pub struct Ed25519KeyMaterial {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl Ed25519KeyMaterial {
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        let verifying_key = signing_key.verifying_key();
        Self { signing_key, verifying_key }
    }
    
    pub fn from_seed(seed: [u8; 32]) -> Result<Self, Ed25519Error> {
        let signing_key = SigningKey::from_bytes(&seed);
        let verifying_key = signing_key.verifying_key();
        Ok(Self { signing_key, verifying_key })
    }
    
    pub fn sign(&self, data: &[u8]) -> Result<[u8; 64], Ed25519Error> {
        let signature = self.signing_key.sign(data);
        Ok(signature.to_bytes())
    }
    
    pub fn verify(&self, data: &[u8], signature: &[u8; 64]) -> Result<bool, Ed25519Error> {
        let sig = Signature::from_bytes(signature);
        Ok(self.verifying_key.verify(data, &sig).is_ok())
    }
    
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }
}
```

```szdt_rust/src/wasm/ed25519_wrapper.rs#L1-70
// WASM wrapper with JS-friendly error handling
use wasm_bindgen::prelude::*;
use crate::ed25519_core::{Ed25519KeyMaterial as CoreKeyMaterial, Ed25519Error};

#[wasm_bindgen]
pub struct Ed25519KeyMaterial {
    inner: CoreKeyMaterial,
}

// Convert core errors to JsError
impl From<Ed25519Error> for JsError {
    fn from(err: Ed25519Error) -> Self {
        match err {
            Ed25519Error::InvalidSeed => JsError::new("Invalid seed: must be 32 bytes"),
            Ed25519Error::InvalidSignature => JsError::new("Invalid signature format"),
            Ed25519Error::SigningFailed => JsError::new("Failed to sign data"),
            Ed25519Error::InvalidMnemonic(msg) => JsError::new(&format!("Invalid mnemonic: {}", msg)),
        }
    }
}

#[wasm_bindgen]
impl Ed25519KeyMaterial {
    #[wasm_bindgen]
    pub fn generate() -> Ed25519KeyMaterial {
        Self {
            inner: CoreKeyMaterial::generate(),
        }
    }
    
    #[wasm_bindgen]
    pub fn from_seed(seed: &[u8]) -> Result<Ed25519KeyMaterial, JsError> {
        if seed.len() != 32 {
            return Err(JsError::new(&format!(
                "Seed must be exactly 32 bytes, got {}", seed.len()
            )));
        }
        
        let mut seed_array = [0u8; 32];
        seed_array.copy_from_slice(seed);
        
        let inner = CoreKeyMaterial::from_seed(seed_array)?;
        Ok(Self { inner })
    }
    
    #[wasm_bindgen]
    pub fn from_mnemonic(mnemonic: &str) -> Result<Ed25519KeyMaterial, JsError> {
        // Additional WASM-specific validation
        if mnemonic.trim().is_empty() {
            return Err(JsError::new("Mnemonic cannot be empty"));
        }
        
        // Call your mnemonic parsing logic
        let seed = crate::mnemonic::mnemonic_to_seed(mnemonic)
            .map_err(|e| JsError::new(&format!("Mnemonic error: {}", e)))?;
            
        Self::from_seed(&seed)
    }
    
    #[wasm_bindgen]
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, JsError> {
        if data.is_empty() {
            return Err(JsError::new("Cannot sign empty data"));
        }
        
        let signature = self.inner.sign(data)?;
        Ok(signature.to_vec())
    }
    
    #[wasm_bindgen]
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, JsError> {
        if signature.len() != 64 {
            return Err(JsError::new("Signature must be 64 bytes"));
        }
        
        let mut sig_array = [0u8; 64];
        sig_array.copy_from_slice(signature);
        
        Ok(self.inner.verify(data, &sig_array)?)
    }
    
    #[wasm_bindgen]
    pub fn public_key(&self) -> Vec<u8> {
        self.inner.public_key_bytes().to_vec()
    }
}
```

## Project Structure with Wrappers

```szdt_rust/src/lib.rs#L1-25
// Core modules - no WASM dependencies
pub mod hash;
pub mod ed25519_key_material; 
pub mod memo;
pub mod did;

// WASM-specific module
#[cfg(target_arch = "wasm32")]
pub mod wasm {
    pub mod hash_wrapper;
    pub mod ed25519_wrapper;
    pub mod memo_wrapper;
    
    // Re-export for easy use
    pub use hash_wrapper::Hash;
    pub use ed25519_wrapper::Ed25519KeyMaterial;
    pub use memo_wrapper::Memo;
}

// CLI-specific modules
#[cfg(not(target_arch = "wasm32"))]
pub mod cli {
    pub mod archive;
    pub mod storage;
}
```

## Benefits of the Wrapper Approach

### 1. **Clean Separation of Concerns**
- Core types remain focused on business logic
- WASM wrappers handle JS-specific concerns (error conversion, type coercion)
- CLI code can use core types directly

### 2. **Better Error Handling**
- Core types can use rich Rust error types
- Wrappers convert to appropriate `JsError`s with user-friendly messages
- No pollution of core error handling with JS concerns

### 3. **API Flexibility**
- Different APIs for different platforms
- WASM API can be more restrictive (validation) or more permissive
- JavaScript-friendly naming and patterns

### 4. **Easier Testing**
- Test core logic independently 
- Test WASM wrappers separately for JS-specific concerns
- Mock/stub easier with clean interfaces

### 5. **Maintainability**
- Changes to WASM API don't affect core types
- Core types can evolve independently
- Easier to add new target platforms later

## When to Use Each Approach

**Use Direct Exposure when:**
- Simple library with minimal API surface
- No significant differences needed between platforms  
- Prototyping or getting started quickly

**Use Wrapper Types when:**
- Complex error handling requirements
- Different APIs needed for different platforms
- Long-term maintainability is important
- Need to add validation specific to WASM/JS context
- Want to keep core types clean and focused

For your SZDT project, I'd strongly recommend the wrapper approach given the complexity of cryptographic operations and the need for robust error handling in a JavaScript environment!

