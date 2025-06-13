# SZDT Specification

**S**igned **Z**ero-trust **D**a**T**a. Pronounced "Samizdat".

Signed CBOR sequences for censorship-resistant data archiving and publishing.

## Motivation

Web resources are accessed by URLs (Uniform Resource Locators), meaning they belong to a single canonical location, or "origin". Security on the web is also [dependent upon origin](https://developer.mozilla.org/en-US/docs/Web/Security/Same-origin_policy), making [mirroring](https://en.wikipedia.org/wiki/Mirror_site) difficult.

All of this means web content is effectively centralized. Centralization makes web data vulnerable to lock-in, censorship, and more banal forms of failure, like [link rot](https://en.wikipedia.org/wiki/Link_rot). A website is a [Single Point of Failure](https://en.wikipedia.org/wiki/Single_point_of_failure) (SPOF), and single points of failure fail eventually. The only question is when.

These limitations may not be a problem for some kinds of content (private messaging, corporate apps), but they become pressing in the case of archival information and publishing. For example, scientific datasets, journalism, reference information, libraries, academic journals, etc are often intended to be broadly accessible public goods, available in perpetuity. However, the websites hosting them can and do disappear, as in the 2025 case of [CDC datasets being taken down by the US government](https://www.theatlantic.com/health/archive/2025/01/cdc-dei-scientific-data/681531/). They may also be subject to censorship in many contexts.

To maintain a resilient information ecosystem, we need a simple way to publish and archive information that:

- Is decentralized, redundant, and censorship-resistant
- Keeps long-tail content alive over long periods of time
- Is easy to adopt **right now**, with infrastructure that is already widely deployed.

## Goals

- **Zero-trust**: SZDT archives are verified using cryptographic hashing and public key cryptography. No centralized authorities are required.
- **Decentralized**: [Lots Of Copies Keeps Stuff Safe](https://www.lockss.org/). SZDT archives are made to be distributed to many redundant locations, including multiple HTTP servers, BitTorrent, hard drives, etc.
- **Censorship-resistant**: Distributable via HTTP, Torrents, email, airdrop, sneakernet, or anything else.
- **Anonymous/pseudonymous**: SZDT uses [keys, not IDs](https://newsletter.squishy.computer/i/60168330/keys-not-ids-toward-personal-illegibility). No accounts are required.
- **Streaming verification**: Blake3 and Bao enable streaming verification of archive integrity without buffering entire files.
- **Random access**: Optional manifests provide efficient seeking to specific content via HTTP range requests or file seeks.

### Non-goals

- **P2P**: SZDT is transport-agnostic. It's just a file format.
- **Efficiency**: SZDT prioritizes simplicity and resilience over efficiency.
- **Comprehensive preservation**: SZDT aims to make it easy to spread data like dandelion seeds.

## Archive Format

SZDT archives are [CBOR sequences](https://www.rfc-editor.org/rfc/rfc8742.html) of **memos** — CBOR objects containing metadata and bytes. Everything in SZDT is represented as a memo.

```
archive = memo1 ‖ memo2 ‖ memo3 ‖ ...
```

Each memo is an independent definite-length CBOR-encoded object. The memos are simply concatenated, one after the other, to make an archive. Since CBOR values are self-delimiting, they can be [concatenated into sequences](https://www.rfc-editor.org/rfc/rfc8742.html) in this way without additional framing.

### Memo Structure

Each memo is a CBOR array containing exactly two elements: `[headers, body]`

```
memo = [headers, body]
```

Where:
- **headers**: CBOR map containing metadata and claims
- **body**: CBOR value representing the content (typically bytes)

### Header Schema

Headers are CBOR maps of open-ended key-value metadata. As with HTTP headers, authors are free to extend headers with additional fields. Also like HTTP headers, some headers have special meanings defined by the protocol, and in some cases, may be required.

#### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `iss` | String | Issuer DID (Decentralized Identifier) |
| `iat` | Integer | Issued at timestamp (Unix seconds) |
| `digest` | Bytes(32) | Blake3 hash of the body content |

#### Optional Fields

| Field | Type | Description |
|-------|------|-------------|
| `nbf` | Integer | Not valid before timestamp (Unix seconds) |
| `exp` | Integer | Expiration timestamp (Unix seconds) |
| `prev` | Bytes(32) | Blake3 hash of previous version of this memo |
| `content-type` | String | MIME content type of body |
| `path` | String | File path for this resource |
| `sig` | Bytes | Ed25519 signature over headers (excluding sig field) |

#### Additional Fields

Headers may contain additional fields for application-specific metadata. To avoid confusion, header field keys should be lowercase strings.

#### Example Memo

```cbor
[
  {
    "iss": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "iat": 1640995200,
    "nbf": 1640995200,
    "digest": h'1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
    "ctype": "text/plain",
    "path": "/example.txt",
    "sig": h'abcd1234...'
  },
  h'48656c6c6f20576f726c64'  // "Hello World"
]
```

### CBOR Encoding

All CBOR items MUST use the deterministic [CBOR/c ("CBOR Core")](https://datatracker-ietf-org.lucaspardue.com/doc/draft-rundgren-cbor-core/) profile with definite-length encoding to ensure deterministic serialization for hashing and signing.

### Content Addressing

All content is addressed using Blake3 hashes.

Memo body integrity verified by comparing `digest` hash field with Blake3 hash of body content. Since digests are Blake3 hashes, they can support streaming verification using Blake3/Bao.

## Signatures

Memos may optionally be signed to enable zero-trust verification of authenticity.

The signature happens over the memo headers. Only the headers are protected by the signature, not the body bytes. To sign over the body bytes, we may include a `digest` field in the headers that with the Blake3 hash of the body content. This approach allows headers to be distributed separately from the body bytes as proofs. This can be useful in scenarios where the body bytes may be distributed separately (e.g. through a content addressed protocol). It also makes verifying the signature cheaper, since we only need to copy the headers when reconstructing the unsigned memo.

### Signing Process

1. Create memo with all fields except `sig`
2. CBOR-encode the memo using CBOR/c profile
3. Compute Blake3 hash of encoded memo
4. Sign hash using Ed25519 private key corresponding to the `iss` DID
5. Set `sig` field to signature bytes

### Verification Process

1. Extract signature from `sig` field
2. Create copy of memo with `sig` field removed
3. CBOR-encode the unsigned memo using CBOR/c profile
4. Compute Blake3 hash of encoded memo
5. Verify the signature over this hash using Ed25519 public key from `iss` DID
6. Verify body integrity by comparing `digest` hash with Blake3 hash of body content

### DID Key Format

Issuers are identified using `did:key` format with Ed25519 keys:

```
did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK
```

The public key can be extracted from the DID for signature verification.

There is planned support for other DID methods:

- `did:web`
- TODO

## Manifests

Optional manifests enable efficient random access to archive contents.

### Manifest Structure

Manifests are memos with content type `application/vnd.szdt.manifest+cbor`:

```cbor
[
  {
    "iss": "did:key:...",
    "iat": 1640995200,
    "digest": h'...',  // Blake3 hash of manifest entries
    "ctype": "application/vnd.szdt.manifest+cbor"
  },
  {
    "digest": h'...',  // Blake3 hash of manifest sequence
    "entries": [
      {
        "length": 150, // Length of memo in bytes
        "digest": h'1234567890abcdef...',
        "path": "/example.txt", // File path
      },
      // ... more entries
    ]
  }
]
```

Manifests are always constructed in relation to some sequence, and entries must appear in the same order as the corresponding memos of that sequence.

### Manifest Entry Schema

| Field | Type | Description |
|-------|------|-------------|
| `path` | String | File path within archive |
| `length` | Integer | Memo length in bytes |
| `digest` | Bytes(32) | Blake3 hash of complete memo |

### Manifest Distribution

Manifests can be distributed:
- **Embedded**: As the first memo in the archive
- **Sidecar**: Generated or distributed independently

## Random Access

With a manifest, any memo can be accessed efficiently:

1. **Locate**: Find desired path or digest in manifest
2. **Calculate offset**: Sum lengths of all preceding memos
3. **Seek**: Navigate to calculated byte offset
4. **Read**: Extract `length` bytes from offset
5. **Verify**: Compare Blake3 hash with manifest digest

### HTTP Range Requests

```http
GET /archive.szdt HTTP/1.1
Range: bytes=1024-2048
```

### File System Seeking

```rust
file.seek(SeekFrom::Start(offset))?;
let mut buffer = vec![0; length];
file.read_exact(&mut buffer)?;
```

## Streaming Verification

SZDT supports streaming verification using Blake3's tree structure:

1. **Progressive hashing**: Compute Blake3 hash incrementally as bytes arrive
2. **Early verification**: Detect tampering before complete download
3. **Bao integration**: Use Bao for efficient partial verification of large content

## Content Types

Memos support open-ended metadata and content. In the context of SZDT archives, memos commonly use the following content types:

| Content Type | Description | Body Format |
|--------------|-------------|-------------|
| `application/octet-stream` | Raw binary data | Bytes |
| `application/vnd.szdt.manifest+cbor` | Archive manifest | CBOR-encoded manifest |

## Versioning and Updates

Memos can reference previous versions using the `prev` field:

```cbor
{
  "iss": "did:key:...",
  "iat": 1640998800,
  "prev": h'abcd1234...', // Blake3 hash of previous version
  "body": h'5678efab...',
  // ... other fields
}
```

This creates a hash-linked chain of versions for the same logical content, enabling Git-like versioning of resources.

## Security Considerations

### Timestamp Validation

- Verify `iat` (issued at) is not in the future
- Respect `nbf` (not before) and `exp` (expires) if present
- Allow reasonable clock skew tolerance

### Hash Collision Resistance

- Blake3 provides strong collision resistance
- All hashes are 256-bit (32 bytes)
- Content addressing prevents tampering

### Signature Security

- Ed25519 provides strong signature security
- Each memo is independently signed
- Verification requires no external dependencies

## Implementation Notes

### CBOR Libraries

Implementations MUST support:
- CBOR/c deterministic encoding
- Streaming CBOR parsing for large archives
- Fixed-length encoding for predictable performance

### Blake3 Libraries

Implementations SHOULD support:
- Incremental hashing for streaming
- Bao for efficient partial verification
- Multi-threading for large content

### HTTP Compatibility

Archives SHOULD be served with:
- `Content-Type: application/vnd.szdt+cbor`
- `Accept-Ranges: bytes` for random access
- Appropriate CORS headers for browser access

## Examples

### Simple Text File Archive

```cbor
// Memo 1: Text file
[
  {
    "iss": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "iat": 1640995200,
    "body": h'a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3',
    "ctype": "text/plain",
    "path": "/hello.txt",
    "sig": h'5d2f8a0f...'
  },
  h'48656c6c6f20576f726c64'  // "Hello World"
]
```

### Archive with Manifest

```cbor
// Memo 1: Manifest
[
  {
    "iss": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "iat": 1640995200,
    "digest": h'b7e2c3d4...',
    "ctype": "application/vnd.szdt.manifest+cbor",
    "sig": h'5d2f8a0f...'
  },
  {
    "entries": [
      {
        "path": "/hello.txt",
        "length": 89,
        "digest": h'c8d5e6f7...'
      }
    ]
  }
]

// Memo 2: Text file
[
  {
    "iss": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "iat": 1640995200,
    "digest": h'a665a459...',
    "ctype": "text/plain",
    "path": "/hello.txt"
  },
  h'48656c6c6f20576f726c64'
]
```

## References

- [CBOR Sequences (RFC 8742)](https://www.rfc-editor.org/rfc/rfc8742.html)
- [CBOR Core Profile](https://datatracker-ietf-org.lucaspardue.com/doc/draft-rundgren-cbor-core/)
- [Blake3 Specification](https://github.com/BLAKE3-team/BLAKE3-specs)
- [Bao Specification](https://github.com/oconnor663/bao)
- [DID Key Method](https://w3c-ccg.github.io/did-method-key/)
- [Ed25519 Signature Scheme](https://datatracker.ietf.org/doc/html/rfc8032)

## Appendix

### MIME Type

SZDT archives SHOULD use the MIME type:

```
application/vnd.szdt.archive+cbor-seq
```

### File Extension

SZDT archives SHOULD use the file extension:

```
.szdt
```

### Interoperability

SZDT is designed to be:
- Transport agnostic (HTTP, BitTorrent, sneakernet, etc.)
- Platform independent (browsers, servers, mobile, etc.)
- Language agnostic (implementable in any language with CBOR support)
