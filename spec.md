# SZDT Specification

**S**igned **Z**ero-trust **D**a**T**a. Pronounced "Samizdat".

Signed CBOR for censorship-resistant data archives.

## Motivation

Web resources are accessed by URLs (Uniform Resource Locators), meaning they belong to a single canonical location, or "origin". Security on the web is also [dependent upon origin](https://developer.mozilla.org/en-US/docs/Web/Security/Same-origin_policy), making [mirroring](https://en.wikipedia.org/wiki/Mirror_site) difficult.

All of this means web content is effectively centralized. Centralization makes web data vulnerable to lock-in, censorship, and more banal forms of failure, like [link rot](https://en.wikipedia.org/wiki/Link_rot). A website is a [Single Point of Failure](https://en.wikipedia.org/wiki/Single_point_of_failure) (SPOF), and single points of failure fail eventually. The only question is when.

These limitations become pressing in the case of archival data. Scientific datasets, libraries, journals, are often intended to be broadly accessible public goods, available in perpetuity. However, the websites hosting them may go down, or be subject to censorship, as in the 2025 case of [CDC datasets being taken down by the US government](https://www.theatlantic.com/health/archive/2025/01/cdc-dei-scientific-data/681531/).

To maintain a resilient information ecosystem, we need a simple way to publish and archive information that:

- Is decentralized and censorship-resistant
- Keeps long-tail content alive over long periods of time
- Is easy to adopt **right now**, with infrastructure that is already widely deployed.

## Goals

- **Zero-trust**: SZDT archives are verified using cryptographic hashing and public key cryptography. No centralized authorities are required.
- **Decentralized**: [Lots Of Copies Keeps Stuff Safe](https://www.lockss.org/). SZDT archives are made to be distributed to many redundant locations, including multiple HTTP servers, BitTorrent, hard drives, etc.
- **Censorship-resistant**: Distributable via HTTP, Torrents, email, airdrop, sneakernet, or anything else.
- **Anonymous/pseudonymous**: SZDT uses [keys, not IDs](https://newsletter.squishy.computer/i/60168330/keys-not-ids-toward-personal-illegibility). No accounts are required.
- **Resilient**: built on CBOR sequences, an [IETF standard](https://cbor.io/spec.html) that is widely supported and should be readable 10, 20, 100 years into the future.
- **Streamable**: CBOR is inherently streamable, and Blake3 hashes enable streaming cryptographic verification.

### Non-goals

- **P2P**: SZDT is transport-agnostic. It's just a file format.
- **Efficiency**: SZDT prioritizes simplicity and resilience over efficiency.
- **Comprehensive preservation**: SZDT aims to make it easy to spread data like dandelion seeds.

## Archive Format

SZDT archives are [CBOR sequences](https://www.rfc-editor.org/rfc/rfc8742.html) of definite-length CBOR values, with the following high-level structure:

```
archive = memo | manifest | bytes | bytes | ...
```

Where:
- **memo**: A signed memo (metadata structure) describing and pointing to a manifest that immediately follows it.
- **manifest**: A CBOR manifest listing all resources in the archive
- **bytes**: the raw bytes for each resource, in the same order as they appear in the manifest.

### Memo Structure

A memo are metadata envelopes that conceptually "wrap" body content, allowing you to annotate the body with additional metadata, as well as cryptographically sign it.

Memos are a general-purpose data structure that takes inspiration from [the flexible memo format used in HTTP and email](https://newsletter.squishy.computer/p/if-headers-did-not-exist-it-would). SZDT uses memos to sign data archives, but they can also be used to annotate and cryptographically sign arbitrary data.

Each memo is a CBOR object containing exactly two elements:

```
memo = {
  unprotected: unprotected_headers,
  protected: protected_headers
}
```

Where:
- **unprotected_headers**: A CBOR map containing key-value metadata. Unprotected headers are not covered by cryptographic signatures, and may be freely modified by anyone.
- **protected_headers**: CBOR map containing key-value metadata. When signing a memo, protected headers are covered by the cryptographic signature.

The memo CBOR does not actually wrap the body content, but instead points to it via a Blake3 hash in the `protected.src` field. This allows the header signature to be validated immiediately, while the body may be streaming-verified using Blake3/Bao. It also makes it possible to distribute the headers and body separately (e.g. via content addressing).

### Header Schema

Both protected and unprotected headers are CBOR maps of open-ended key-value metadata. As with HTTP headers, authors are free to extend these headers with additional fields. Also like HTTP headers, some headers have special meanings defined by the protocol. Certain headers may be required.

#### Required unprotected headers

None.

#### Optional protected headers

| Field | Type | Description |
|-------|------|-------------|
| `sig` | Bytes | An Ed25519 cryptographic signature over the protected headers |

Optional headers that are not given definite values MUST be omitted when serializing.

#### Required protected headers

| Field | Type | Description |
|-------|------|-------------|
| `iss` | String | Issuer DID (Decentralized Identifier) (required for signed memos) |
| `iat` | Integer | Issued at timestamp (Unix seconds) |
| `src` | Bytes(32) | Blake3 hash of the content this memo points to |

#### Optional Fields

| Field | Type | Description |
|-------|------|-------------|
| `nbf` | Integer | Not valid before timestamp (Unix seconds) |
| `exp` | Integer | Expiration timestamp (Unix seconds) |
| `prev` | Bytes(32) | Blake3 hash of previous version of this memo |
| `content-type` | String | MIME content type of body |
| `path` | String | File path for this resource |

Optional headers that are not given definite values MUST be omitted when serializing.

#### Additional Fields

Both protected and unprotected headers may contain open-ended additional fields for application-specific metadata.

To avoid confusion, header field keys (including headers borrowed from HTTP) should always be lowercase strings.

When defining custom headers, `protected` headers should be preferred, unless the use-case requires the header not to be cryptographically protected.

Headers defined in HTTP suite of specifications must be considered to have the same semantics as in HTTP. Applications should not define headers that conflict with the semantics of HTTP headers.

#### Example Memo

```cbor
{
  "protected": {
    "iss": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "iat": 1640995200,
    "nbf": 1640995200,
    "src": h'1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
    "content-type": "application/vnd.szdt.manifest+cbor",
  },
  "unprotected": {
    "sig": h'abcd1234...'
  },
}
```

### CBOR Encoding

All CBOR items MUST use the deterministic [CBOR/c ("CBOR Core")](https://datatracker-ietf-org.lucaspardue.com/doc/draft-rundgren-cbor-core/) profile with definite-length encoding to ensure deterministic serialization for hashing and signing.

### Content Addressing

All content is addressed using Blake3 hashes.

Content integrity is verified by comparing the `src` hash field in memo protected headers with the Blake3 hash of the referenced content. Since `src` values are Blake3 hashes, they can support streaming verification using Blake3/Bao.

Content addressing enables:
- **Separation of signatures from content**: Signatures can be distributed independently from the content they authenticate
- **Deduplication**: Identical content shares the same hash regardless of where it appears
- **Streaming verification**: Blake3's tree structure allows verification before complete download

## Signatures

Memos may optionally be signed to enable zero-trust verification of authenticity.

Signatures cover only the protected headers of the memo.

### Signing Process

1. Compute the Blake3 hash of the CBOR-encoded protected header map, using the CBOR/c profile.
2. Assign a DID to `protected.iss` pointing to the Ed25519 public key of the keypair being used to sign the memo.
3. Sign the hash using the Ed25519 private key corresponds to the `iss` DID.
4. In the unprotected headers, set the `sig` header to the signature bytes

### Verification Process

1. Extract signature from `sig` field in the unprotected headers
2. Compute Blake3 hash of the CBOR-encoded protected headers, using the CBOR/c profile.
3. Verify the signature over the hash using Ed25519 public key from `iss` DID

Verification proves the authenticity of the memo, but does not verify the integrity of the resource associated with the memo via its `protected.src` field. When reading the memo's body, the additional integrity verification steps should be carried out:

1. Compute the Blake3 hash of the CBOR-encoded body content, using the CBOR/c profile.
2. Compare the computed hash to the hash found in `protected.src`. If hashes are equal, the integrity check has succceeded.

### DID Key Format

Issuers are identified using `did:key` format with Ed25519 keys:

```
did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK
```

The public key can be extracted from the DID for signature verification.

There is planned support for other DID methods:

- `did:web`
- TODO other methods

## Manifests

The SZDT Manifest is a CBOR map describing the archive resources. In an SZDT archive, the manifest is pointed to by a memo with the content type `application/vnd.szdt.manifest+cbor`.

### Manifest Structure

```
{
  "resources": [
    {
      "src": h'1234567890abcdef...',  // Blake3 hash of resource content
      "length": 150,                  // Length of resource in bytes
      "path": "/example.txt",         // File path
      "content-type": "text/plain"    // MIME type (optional)
    },
    // ... more resources
  ]
}
```

Manifests list all resources in the archive. Resources must appear in the same order as the corresponding byte sequences that follow the manifest in the CBOR sequence.

### Manifest Resource Schema

| Field | Type | Description |
|-------|------|-------------|
| `src` | Bytes(32) | Blake3 hash of resource content |
| `length` | Integer | Length in bytes of the CBOR structure pointed to by src |
| `path` | String | File path within archive |
| `content-type` | String | MIME content type (optional) |

## Random Access

Manifests enable efficient random acccess:

1. **Read memo**: Read the first CBOR item (memo) to get manifest hash
2. **Read manifest**: Read the second CBOR item (manifest) and verify against memo's `src` hash
3. **Locate**: Find desired path or hash in manifest resources
4. **Calculate offset**: Sum lengths of all preceding resources, plus size of memo and manifest
5. **Seek**: Navigate to calculated byte offset
6. **Read**: Extract `length` bytes from offset
7. **Verify**: Compare Blake3 hash with manifest resource `src` hash

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

Memos can reference previous versions using the `prev` field in protected headers:

```cbor
{
  "unprotected": {
    "sig": h'9876fedc...'
  },
  "protected": {
    "iss": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "iat": 1640998800,
    "prev": h'abcd1234...', // Blake3 hash of previous version memo
    "src": h'5678efab...',  // Blake3 hash of current content
    "content-type": "application/vnd.szdt.manifest+cbor"
  }
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

```
// Manifest memo
{
  "unprotected": {
    "sig": h'5d2f8a0f...'
  },
  "protected": {
    "iss": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "iat": 1640995200,
    "src": h'a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3',
    "content-type": "application/vnd.szdt.manifest+cbor"
  }
}

// Manifest
{
  "resources": [
    {
      "src": h'c8d5e6f7...',
      "length": 11,
      "path": "/hello.txt",
      "content-type": "text/plain"
    }
  ]
}

// Resource bytes
h'48656c6c6f20576f726c64'  // "Hello World"
```

### Archive with Multiple Files

```cbor
// Memo: Manifest wrapper
{
  "unprotected": {
    "sig": h'5d2f8a0f...'
  },
  "protected": {
    "iss": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "iat": 1640995200,
    "src": h'b7e2c3d4...',
    "content-type": "application/vnd.szdt.manifest+cbor"
  }
}

// Manifest
{
  "resources": [
    {
      "src": h'c8d5e6f7...',
      "length": 11,
      "path": "/hello.txt",
      "content-type": "text/plain"
    },
    {
      "src": h'a1b2c3d4...',
      "length": 25,
      "path": "/data.json",
      "content-type": "application/json"
    }
  ]
}

// Resource 1 bytes
h'48656c6c6f20576f726c64'  // "Hello World"

// Resource 2 bytes
h'7b226b6579223a2276616c7565227d'  // {"key":"value"}
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
