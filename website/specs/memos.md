---
layout: index.liquid
title: SZDT Memos Specification
---

# SZDT Memos Specification

Memos are general-purpose metadata envelopes for annotating and cryptographically signing arbitrary CBOR data.

Memos take inspiration from [the flexible memo format used in HTTP and email](https://newsletter.squishy.computer/p/if-headers-did-not-exist-it-would), with headers encoding arbitrary key-value metadata, followed by a body.

Memos provide a standardized way to:

- **Annotate data** with open-ended key-value metadata.
- **Cryptographically sign data** to create self-certifying records.
- **Address data** using cryptographic hashes for integrity verification.
- **Version data** using Git-like semantics.

## Memo Structure

Memos are CBOR maps containing headers divided into two buckets: `protected` and `unprotected`.

An example of a basic memo:

```cbor
{
  "type": "szdt/memo", // always "szdt/memo"
  "protected": {
    "iss": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "iat": 1640995200,
    "src": h'c8d5e6f7a9b8c7d6e5f4g3h2i1j0k9l8m7n6o5p4q3r2s1t0u9v8w7x6y5z4a3b2c1',
    "content-type": "text/plain"
  },
  "unprotected": {
    "sig": h'5d2f8a0f3b4c7e8f9a1b2c3d4e5f6789abcdef01234567890abcdef0123456789abcdef01234567890abcdef0123456789abcdef01234567890abcdef'
  }
}
```

- **protected**: A CBOR map of headers (key-value metadata). Protected headers are covered by cryptographic signatures when the memo is signed. Protected headers contain security-critical metadata like content hashes, timestamps, and issuer information.
- **unprotected**: A CBOR map of headers (key-value metadata). Unprotected headers are not covered by cryptographic signatures and may be freely modified. Unprotected headers contain auxiliary metadata, such as the signatures themselves, caching hints, or routing information.

This design allows intermediaries to add metadata (like caching information) without invalidating signatures, while critical metadata remains tamperproof.

## Content Addressing

SZDT memos may point to other resources using _content addresses_ (referencing data by cryptographic hash). SZDT content addresses are always [Blake3 hashes](https://www.ietf.org/archive/id/draft-aumasson-blake3-00.html), serialized in CBOR as byte strings.

Content addressing has a number of benefits:

- **Decentralized**: Content addresses allow us to refer to data based on _what_ it is, not _where_ it lives. Data referenced by a content address may live in multiple locations, and be retreived over multiple transports (HTTP, [Iroh](https://www.iroh.computer/), etc).
- **Zero-trust**: Content addressed data may be retreived from untrusted locations, since the hash can be used to guarantee that the data you asked for is the data you got.
- **Efficient**: Content addresses allow for efficient storage and retrieval of data. Only the hash needs to be stored and transmitted, rather than the entire data payload. The same data can be referenced by multiple memos without duplicating the data itself.

## Memo body

Like HTTP or email, memos are made up of a header part and a body part. Unlike HTTP and email, the body of a memo is not embedded directly into the memo structure itself. Instead, the body is referred to by a content address (Blake3 hash), stored in the `src` field of the protected headers.

```cbor
{
  "type": "szdt/memo",
  "unprotected": ...,
  "protected": {
    src: h'abcd1234...',
    ...
  }
}
```

This design enables:

- **Streaming verification**: The signature over the headers may be verified immediately, while the body part is be streamed and verified incrementally via Blake3/[Bao](https://github.com/oconnor663/bao/blob/master/docs/spec.md).
- **Flexible delivery**: Because the signature happens over the headers containing the hash of the body, the body part can be delivered in multiple ways. Memo and body can be bundled together into an [SZDT sequence](./sequence/), or distributed independently.

## Required and optional headers

Both protected and unprotected headers are CBOR maps containing open-ended key-value metadata.

Some header keys have predefined semantics. Some header keys are required, while others are optional. Optional headers must be omitted when not given a definite value. Authors must not serialize null values for unused headers.

## Protected Headers

Required headers:

| Field | Type | Description |
|-------|------|-------------|
| `iss` | String | Issuer DID (required for signed memos) |
| `iat` | Integer | Issued at timestamp (Unix seconds) |
| `src` | Bytes(32) | Blake3 hash of the referenced content |

Optional headers:

| Field | Type | Description |
|-------|------|-------------|
| `nbf` | Integer | Not valid before timestamp (Unix seconds) |
| `exp` | Integer | Expiration timestamp (Unix seconds) |
| `prev` | Bytes(32) | Blake3 hash of previous version of this memo |
| `content-type` | String | MIME content type of referenced content |

## Unprotected Headers

Optional:

| Field | Type | Description |
|-------|------|-------------|
| `sig` | Bytes | Ed25519 cryptographic signature over protected headers |

## Custom Headers

Applications may define additional headers for application-specific use cases. Custom headers:

- Should use lowercase string keys
- Should prefer protected headers, unless the header needs to be modifiable

### Compatibility with HTTP headers

Headers defined in the HTTP suite of specifications should be considered to have the same semantics as their HTTP counterparts. Custom headers must not be defined that conflict with HTTP header semantics.

## Header Serialization Rules

- Optional headers with undefined values MUST be omitted from serialization
- Header maps MUST use deterministic CBOR encoding (CBOR/c profile)
- Header keys MUST be strings
- Header values may be any valid CBOR type

## CBOR Encoding

All CBOR structures MUST use the deterministic [CBOR/c ("CBOR Core")](https://datatracker-ietf-org.lucaspardue.com/doc/draft-rundgren-cbor-core/) profile with definite-length encoding to ensure deterministic serialization for consistent hashing and signing.

## Signatures

Memos support signing with an Ed25519 signatures to provide cryptographic proof of authenticity. Signatures are optional but recommended for zero-trust scenarios.

### Signing Process

1. **Prepare protected headers**: Ensure all required protected headers are present
2. **Encode headers**: Serialize protected headers to CBOR using CBOR/c profile
3. **Hash headers**: Compute the Blake3 hash of the serialized protected headers
4. **Sign hash**: Generate Ed25519 signature over the Blake3 hash
5. **Add signature**: Place signature in `sig` field of unprotected headers. Signature must be serialized as a CBOR byte string.

### Verification Process

1. **Extract signature**: Get signature from `sig` field in unprotected headers
2. **Encode headers**: Serialize protected headers to CBOR using CBOR/c profile
3. **Hash headers**: Compute Blake3 hash of the serialized protected headers
4. **Verify signature**: Validate Ed25519 signature over the hash using issuer's public key
5. **Verify timestamps**: If `nbf` and `exp` are present, check that `nbf` is not in the future and `exp` is not in the past. A slush factor of 1000 milliseconds may be used to account for clock skew.
6. **Verify content integrity**: Verify content integrity using `src` hash, using the steps outlined below.

To verify content integrity:

1. **Read content**: Obtain the content referenced by the memo
2. **Hash content**: Compute Blake3 hash of the content
3. **Compare hashes**: Verify computed hash matches `src` field in protected headers

For large content, implementors may use Blake3's streaming capabilities or [Bao](https://github.com/oconnor663/bao) for incremental verification.

## DIDs (Decentralized IDentifiers)

Actors in SZDT are identified with [DIDs](https://github.com/w3c/did-wg/blob/main/did-explainer.md). DIDs provide a decentralized way to reference a public key.

Memo issuers (authors) are identified using a [`did:key`](https://w3c-ccg.github.io/did-key-spec/) encoding the Ed25519 public key that may be used to verify the memo signature. The issuer DID is stored on the `iss` field of the protected headers.

```
{
  "iss": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
}
```

DIDs are always serialized to their string representation.

Future versions of SZDT may support additional DID methods, such as [`did:web`](https://w3c-ccg.github.io/did-method-web/).


## Versioning and Updates

Memos can reference previous versions using the `prev` field in protected headers:

```cbor
{
  "type": "szdt/memo",
  "unprotected": {
    "sig": h'9876fedc...'
  },
  "protected": {
    "iss": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "iat": 1640998800,
    "prev": h'abcd1234...', // Blake3 hash of previous memo
    "src": h'5678efab...',  // Blake3 hash of current content
    "content-type": "text/plain"
  }
}
```

This creates a hash-linked chain of versions enabling Git-like version history.

Applications may choose to interpret this version history in a variety of ways, such as displaying a timeline of changes, or implementing branching workflows.

For example, to determine the most recent version, an application might implement last-write wins semantics using the following scheme:

- Choose a `iss` (issuer) to trust. Of the memos issued by that issuer...
  - Compare `iat` timestamps of versions. Newest wins.
    - If more than one memo has the newest `iat` timestamp, take the Blake3 hashes of the conflicting memos, and sort them in bytewise lexicographic order. The largest hash is the most recent version.

Applications are also free to implement other versioning strategies, such as comparing the longest branch from a common ancestor, embedding CRDTs in the body, etc.

## Usage Examples

### Basic Signed Memo

```cbor
{
  "type": "szdt/memo",
  "unprotected": {
    "sig": h'5d2f8a0f3b4c7e8f9a1b2c3d4e5f6789abcdef01234567890abcdef0123456789abcdef01234567890abcdef0123456789abcdef01234567890abcdef'
  },
  "protected": {
    "iss": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "iat": 1640995200,
    "src": h'c8d5e6f7a9b8c7d6e5f4g3h2i1j0k9l8m7n6o5p4q3r2s1t0u9v8w7x6y5z4a3b2c1',
    "content-type": "text/plain"
  }
}
```

### Versioned Content Chain

```cbor
// Version 1
{
  "type": "szdt/memo",
  "unprotected": {
    "sig": h'signature1...'
  },
  "protected": {
    "iss": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "iat": 1640995200,
    "src": h'content_hash_v1...',
    "content-type": "application/json"
  }
}

// Version 2 (references version 1)
{
  "type": "szdt/memo",
  "unprotected": {
    "sig": h'signature2...'
  },
  "protected": {
    "iss": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "iat": 1640998800,
    "prev": h'memo_hash_v1...',
    "src": h'content_hash_v2...',
    "content-type": "application/json"
  }
}
```

## References

- [CBOR Core Profile](https://datatracker-ietf-org.lucaspardue.com/doc/draft-rundgren-cbor-core/)
- [Blake3 Specification](https://github.com/BLAKE3-team/BLAKE3-specs)
- [Ed25519 Signature Scheme (RFC 8032)](https://datatracker.ietf.org/doc/html/rfc8032)
- [DID Key Method](https://w3c-ccg.github.io/did-key-spec/)
- [HTTP Semantics (RFC 9110)](https://datatracker.ietf.org/doc/html/rfc9110)

## Appendix

### MIME Type

SZDT memos should use the MIME type:

```
application/vnd.szdt.memo+cbor
```
