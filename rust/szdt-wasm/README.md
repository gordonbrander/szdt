# SZDT WASM

WebAssembly bindings for SZDT core functionality, providing cryptographic operations, memo management, and data structures for JavaScript/TypeScript applications.

## Features

- **Hash Operations**: Blake3 hashing with encoding support
- **Ed25519 Key Management**: Key generation, signing, verification, and DID conversion
- **Memo Operations**: Create, sign, validate, and serialize memos
- **DID Key Support**: Parse and manipulate DID:key identifiers
- **Mnemonic Support**: BIP39 mnemonic phrase handling
- **CBOR Sequences**: Read and write CBOR sequence data

## Building

### Prerequisites

Install `wasm-pack`:

```bash
cargo install wasm-pack
```

### Build for Web

```bash
wasm-pack build --target web --out-dir pkg-web
```

### Build for Node.js

```bash
wasm-pack build --target nodejs --out-dir pkg-node
```

### Build for Bundlers (Webpack, etc.)

```bash
wasm-pack build --target bundler --out-dir pkg-bundler
```

## Usage Example

```typescript
import { Hash, Ed25519KeyMaterial, Memo } from './pkg-web/szdt_wasm.js';

// Create a hash
const data = new Uint8Array([1, 2, 3, 4]);
const hash = new Hash(data);
console.log(hash.toString()); // Base32 representation

// Generate keys
const keyMaterial = Ed25519KeyMaterial.generate();
console.log(keyMaterial.did_string());

// Create and sign a memo
const memo = new Memo(hash);
memo.sign(keyMaterial);
const isValid = memo.verify();
console.log('Memo is valid:', isValid);

// Serialize to CBOR
const cborData = memo.to_cbor();
```

## API Reference

### Hash
- `new Hash(data: Uint8Array)` - Create hash from data
- `Hash.from_bytes(bytes: Uint8Array)` - Create from hash bytes
- `Hash.from_string(input: string)` - Create from string
- `as_bytes(): Uint8Array` - Get hash bytes
- `toString(): string` - Get base32 representation
- `equals(other: Hash): boolean` - Compare hashes

### Ed25519KeyMaterial
- `Ed25519KeyMaterial.generate()` - Generate new key pair
- `Ed25519KeyMaterial.from_seed(seed: Uint8Array)` - From 32-byte seed
- `Ed25519KeyMaterial.from_mnemonic(mnemonic: string)` - From BIP39 mnemonic
- `Ed25519KeyMaterial.from_public_key(pubkey: Uint8Array)` - Public key only
- `public_key(): Uint8Array` - Get public key bytes
- `private_key(): Uint8Array | undefined` - Get private key bytes (if available)
- `did(): DidKey` - Get DID key representation
- `did_string(): string` - Get DID as string
- `sign(data: Uint8Array): Uint8Array` - Sign data
- `verify(data: Uint8Array, signature: Uint8Array): boolean` - Verify signature
- `can_sign(): boolean` - Check if private key is available

### Memo
- `new Memo(bodyHash: Hash)` - Create memo with body hash
- `Memo.for_body(content: Uint8Array)` - Create memo for content
- `Memo.for_string(content: string)` - Create memo for string
- `Memo.empty()` - Create empty memo
- `sign(keyMaterial: Ed25519KeyMaterial)` - Sign the memo
- `verify(): boolean` - Verify signature
- `validate(timestamp?: number): boolean` - Full validation
- `to_cbor(): Uint8Array` - Serialize to CBOR
- `Memo.from_cbor(data: Uint8Array)` - Deserialize from CBOR
- Various getters/setters for metadata (timestamp, expiration, content type, etc.)

### DidKey
- `new DidKey(publicKey: Uint8Array)` - Create from public key
- `DidKey.parse(didKeyUrl: string)` - Parse DID:key URL
- `public_key(): Uint8Array` - Get public key bytes
- `toString(): string` - Get DID:key URL
- `equals(other: DidKey): boolean` - Compare DIDs
- `DidKey.is_valid(didKeyUrl: string): boolean` - Validate DID format

### Mnemonic
- `Mnemonic.from_entropy(entropy: Uint8Array)` - Create from entropy
- `Mnemonic.parse(mnemonic: string)` - Parse mnemonic phrase
- `toString(): string` - Get mnemonic phrase
- `to_entropy(): Uint8Array` - Get entropy bytes
- `word_count(): number` - Get word count
- `Mnemonic.generate_12_word()` - Generate 12-word mnemonic
- `Mnemonic.generate_24_word()` - Generate 24-word mnemonic
- (And 15, 18, 21-word variants)

### CBOR Sequences
- `CborSeqReader` / `CborSeqWriter` - For reading/writing CBOR sequence files
- Utility functions for CBOR parsing and serialization

## Target Compatibility

- Web browsers (with ES modules)
- Node.js
- Bundlers (Webpack, Rollup, etc.)
- TypeScript (with generated .d.ts files)