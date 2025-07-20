# Rust Project Crate Architecture Strategy

## User


<context>
The following items were attached by the user. They are up-to-date and don't need to be re-read.

<conversation_threads>
Rust WebAssembly Dependency Configuration
# WASM Compilation in Rust: Comprehensive Guide

## Overview
This conversation explored the process of compiling a Rust library to WebAssembly (WASM), focusing on best practices, error handling, type exposure, and project structure for cross-platform development.

## Key Topics Discussed
- WASM compilation dependencies
- Conditional compilation techniques
- JavaScript/TypeScript interoperability
- Error handling with `JsError`
- Wrapper type patterns for WASM exposure

## Key Technical Discoveries

### WASM Compilation Dependencies
- Essential dependencies:
  * `wasm-bindgen`
  * `js-sys`
  * `web-sys`
  * `serde-wasm-bindgen`
  * `wee_alloc` (optional)

### Conditional Compilation Techniques
- Use `#[cfg()]` attributes to:
  * Exclude modules
  * Conditionally include functions
  * Provide platform-specific implementations
- Support multiple compilation targets (CLI, WASM, etc.)

### Error Handling
- `JsError` is crucial for converting Rust errors to JavaScript exceptions
- Best practices include:
  * Providing clear error messages
  * Converting domain-specific errors to user-friendly messages
  * Using `Result<T, JsError>` for functions that can fail

### Wrapper Type Pattern
- Recommended approach for WASM exposure
- Benefits include:
  * Clean separation of concerns
  * Flexible error handling
  * Platform-specific API design
  * Easier testing and maintenance

## Detailed Compilation Strategy

### Cargo.toml Configuration
- Add WASM-specific dependencies
- Use conditional compilation for platform-specific code
- Configure library to support multiple compilation targets

### Source Code Structure
- Maintain core logic in platform-agnostic modules
- Create WASM-specific wrapper modules
- Use `#[cfg()]` attributes for conditional compilation

### Error Handling Approach
- Define core error types in domain logic
- Create conversion implementations to `JsError`
- Provide clear, informative error messages for JavaScript consumers

## Recommended Project Structure
```
src/
├── lib.rs          # Main library entry point
├── core/           # Platform-agnostic core logic
│   ├── hash.rs
│   ├── key_material.rs
│   └── ...
├── wasm/           # WASM-specific wrappers
│   ├── hash_wrapper.rs
│   ├── key_material_wrapper.rs
│   └── ...
└── cli/            # CLI-specific implementations
    ├── storage.rs
    └── ...
```

## Action Items
1. Update `Cargo.toml` with WASM dependencies
2. Create wrapper types for core structs
3. Implement `From<CoreError>` for `JsError`
4. Add `#[wasm_bindgen]` attributes to exposure points
5. Install `wasm-pack` for compilation
6. Create TypeScript type definitions
7. Write WASM-specific tests

## Conclusion
The wrapper type pattern provides a robust, flexible approach to exposing Rust libraries to WebAssembly, allowing for clean separation of concerns and platform-specific optimizations.

## Recommended Next Steps
- Prototype a small module using the wrapper approach
- Gradually migrate existing code to the new structure
- Write comprehensive tests for WASM and core logic
- Create documentation for JavaScript consumers

## Learning Resources
- [Rust WASM Book](https://rustwasm.github.io/docs/book/)
- [wasm-bindgen Documentation](https://rustwasm.github.io/wasm-bindgen/)
- [MDN WebAssembly Guide](https://developer.mozilla.org/en-US/docs/WebAssembly)
</conversation_threads>
</context>

If I have a core set of functionality that I want to expose as a CLI, as a Rust library, and as a JS library through WASM, is it better to keep all of that in a single crate, or to create separate crates?

## Agent

<thinking>
This is a great architectural question about organizing a Rust project that needs to support multiple targets: CLI, Rust library, and WebAssembly/JavaScript. Let me think through the pros and cons of each approach.

First, let me explore the existing project to understand the current structure and see what's already there.
</thinking>

**Use Tool: list_directory (toolu_016XbUUrewsYqCzxekiybq77)**
```json
{
  "path": "szdt_rust"
}
```

**Tool Results: toolu_016XbUUrewsYqCzxekiybq77**

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



**Use Tool: read_file (toolu_011ooZBjX6mtrVJi9vEeqK26)**
```json
{
  "path": "szdt_rust/Cargo.toml"
}
```

**Tool Results: toolu_011ooZBjX6mtrVJi9vEeqK26**

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



**Use Tool: list_directory (toolu_018bbCZitdA7mKePfWv6Yeio)**
```json
{
  "path": "szdt_rust/src"
}
```

**Tool Results: toolu_018bbCZitdA7mKePfWv6Yeio**

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



**Use Tool: read_file (toolu_01DA6oJjCVvLjxkLQKh9qhAp)**
```json
{
  "path": "szdt_rust/src/lib.rs"
}
```

**Tool Results: toolu_01DA6oJjCVvLjxkLQKh9qhAp**

pub mod base58btc;
pub mod bytes;
pub mod cbor_seq;
pub mod config;
pub mod contact;
pub mod content_type;
pub(crate) mod db;
pub mod did;
pub mod ed25519;
pub mod ed25519_key_material;
pub mod error;
pub mod file;
pub mod hash;
pub mod hashseq;
pub mod key_storage;
pub mod link;
pub mod memo;
pub mod mnemonic;
pub mod nickname;
pub mod szdt;
pub mod text;
pub mod time;
pub mod value;

## Agent

Based on your current project structure and the requirements you've described, I'd recommend a **multi-crate workspace approach**. Here's why and how to structure it:

## Recommended Approach: Workspace with Multiple Crates

The best approach for your use case is to create a Cargo workspace with separate crates:

```szdt_rust/Cargo.toml#L1-15
[workspace]
members = [
    "szdt-core",     # Core functionality
    "szdt-cli",      # CLI application
    "szdt-wasm",     # WASM bindings
]

[workspace.dependencies]
# Shared dependencies go here
serde = { version = "1.0.219", features = ["derive"] }
blake3 = { version = "1.8.2", features = ["serde"] }
thiserror = "2.0.12"
# ... other shared deps
```

## Benefits of This Approach

### 1. **Clean Separation of Concerns**
- **`szdt-core`**: Pure Rust logic, no platform-specific code
- **`szdt-cli`**: CLI-specific code (argument parsing, file I/O, user interaction)
- **`szdt-wasm`**: WASM bindings and JavaScript interop

### 2. **Targeted Dependencies**
Each crate can have only the dependencies it needs:
- Core crate: minimal dependencies, no CLI or WASM-specific deps
- CLI crate: `clap`, `dialoguer`, `rusqlite`, etc.
- WASM crate: `wasm-bindgen`, `js-sys`, `web-sys`, etc.

### 3. **Independent Versioning**
You can version and release each crate independently:
- Core library might be at v1.2.0
- CLI might be at v1.1.3
- WASM bindings might be at v0.9.1

### 4. **Better Testing**
- Test core logic without platform concerns
- Test CLI functionality in isolation
- Test WASM bindings separately

### 5. **Easier Maintenance**
- Changes to CLI don't affect core library users
- WASM compilation issues don't block CLI development
- Clear ownership of different concerns

## Suggested Project Structure

```/dev/null/structure.txt#L1-25
szdt_rust/
├── Cargo.toml              # Workspace root
├── szdt-core/              # Core functionality
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── hash.rs         # Your current core modules
│   │   ├── ed25519.rs
│   │   └── ...
├── szdt-cli/               # CLI application
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/
│   │   └── storage/        # CLI-specific storage
├── szdt-wasm/              # WASM bindings
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── wrappers/       # WASM wrapper types
│   │   └── utils/
└── examples/               # Shared examples
```

## Migration Strategy

Since you already have a working single-crate structure, here's how to migrate:

### Step 1: Create the workspace structure
```szdt_rust/Cargo.toml#L1-5
[workspace]
members = [
    "szdt-core",
    "szdt-cli",
    "szdt-wasm",
]
```

### Step 2: Move core functionality
Extract your current `src/` modules into `szdt-core/src/`, keeping only the pure Rust logic.

### Step 3: Create CLI crate
Move CLI-specific code (your current `src/bin/szdt.rs` and any CLI-only modules) to `szdt-cli/`.

### Step 4: Add WASM crate
Create the WASM bindings as a new crate that depends on `szdt-core`.

## Alternative: Single Crate with Features

If you prefer to keep everything in one crate, you could use feature flags:

```szdt_rust/Cargo.toml#L1-20
[package]
name = "szdt"
version = "0.0.1"
edition = "2024"

[features]
default = ["cli"]
cli = ["clap", "dialoguer", "rusqlite"]
wasm = ["wasm-bindgen", "js-sys", "web-sys"]

# CLI-specific dependencies
[dependencies.clap]
version = "4.5.31"
features = ["derive"]
optional = true

# WASM-specific dependencies
[dependencies.wasm-bindgen]
version = "0.2"
optional = true
```

However, this approach has downsides:
- All dependencies are still present in `Cargo.toml`
- Harder to version independently
- More complex conditional compilation
- Larger dependency tree for library users

## Recommendation

For your use case, I strongly recommend the **workspace approach**. It provides better separation, cleaner dependency management, and easier maintenance as your project grows. You can start with just `szdt-core` and `szdt-cli`, then add `szdt-wasm` when you're ready to tackle the WebAssembly integration.

Would you like me to help you set up the workspace structure and migrate your existing code?
