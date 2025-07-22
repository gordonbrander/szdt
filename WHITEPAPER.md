# SZDT: signed CBOR for censorship-resistant data

## Summary

**TLDR**: signed CBOR for censorship-resistant data.

SZDT is a simple format for decentralizing data. SZDT data is signed and self-certifying, so it can be distributed across any protocol and cryptographically verified without relying on any centralized authority.

## Problem: websites are single points of failure

On the web, trust is centralized around the server. Web resources are accessed by URLs (Uniform Resource Locators), meaning they belong to a single location. This makes web content centralized, vulnerable to lock-in, link rot, and censorship.

The quick fix would be to distribute data to multiple locations, or even across multiple transports (HTTP, BitTorrent, etc...). This would make data resistant to lock-in, link rot, and censorship. Unfortunately, the web's trust model makes this impossible. Since trust is rooted in the centralized server, we can't trust data we get from other sources. There's no way to know if it has been tampered with.

## Solution: self-certifying data

SZDT solves this by combining two ideas:

- **Public key cryptography**: use cryptographic signatures to trustlessly prove who created the data.
- **Content addressing**: use cryptographic hashes to address data and trustlessly prove data integrity.

Together, these things allow us to create **self-certifying data**, data that can be cryptographically verified without relying on any centralized authority.

We no longer need to trust the server. We can verify with cryptography. Because trust is decoupled from the server, data can be decentralized.

## How It Works

### Memos

SZDT is built around **memos**, CBOR metadata envelopes signed with an Ed25519 cryptographic key.

Memos are conceptually made up of two parts, **headers** and a **body** (the data). This memo format will be familiar if you've worked with HTTP or other internet protocols.

Headers are further broken down into **protected** (covered by signature) and **unprotected** (not covered by signature) headers.

```cbor
{
  "type": "szdt/memo",
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

Each memo contains:

- a **cryptographic signature**, signing over the **protected** data
- a **DID** resolving to the public key used to sign the message
- a **src** containing the **Blake3 hash** of the "body part" of the memo

This gives us everything we need to verify the **authenticity** and **integrity** of the data.

The body bytes are not present in the memo itself. Instead, we use the `src` field, a Blake3 hash of the serialized body bytes, as a **content address**. This enables memos to be distributed separately from content as as proofs. Altneratively, memos and bodies are commonly distributed together as [CBOR sequences](https://www.rfc-editor.org/rfc/rfc8742.html) by simply concatenating the memo, followed by the body part.

Any CBOR value is a valid SZDT value, so bodies can be CBOR byte strings, or more complex CBOR structures.

### Serialization

- SZDT always serializes data using the deterministic [CBOR/c ("CBOR Core")](https://datatracker-ietf-org.lucaspardue.com/doc/draft-rundgren-cbor-core/) profile.
  - The requirements for this serialization profile are the same as IETF RFC 8949, section 4.2.1. ["Core Deterministic Encoding Requirements"](https://datatracker.ietf.org/doc/html/rfc8949#core-det), but add a few additional clarifications to resolve ambiguities.
- Ensures deterministic serialization for consistent hashing and signing.

### Signing scheme

#### Signing

1. **Prepare protected headers**: Ensure all required protected headers are present
2. **Encode headers**: Serialize protected headers to CBOR using CBOR/c profile
3. **Hash headers**: Compute the Blake3 hash of the serialized protected headers
4. **Sign hash**: Generate Ed25519 signature over the Blake3 hash
5. **Add signature**: Place signature in `sig` field of unprotected headers. Signature must be serialized as a CBOR byte string.

#### Verification

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

For big data, implementors may use Blake3's streaming capabilities or [Bao](https://github.com/oconnor663/bao) for incremental verification.

## Key Features

- **Zero-trust**: SZDT archives are verified using cryptographic hashing and public key cryptography. No centralized authorities are required.
- **Censorship-resistant**: Because trust is decoupled from origin or transport, SZDT archives can be distributed via HTTP, Torrents, email, airdrop, sneakernet, or anything else that is available.
- **Decentralizable**: SZDT decouples trust from origin, so data can be distributed to many redundant locations, including multiple HTTP servers, BitTorrent, hard drives, etc. [Lots Of Copies Keeps Stuff Safe](https://www.lockss.org/).
- **Anonymous/pseudonymous**: SZDT uses [keys, not IDs](https://newsletter.squishy.computer/i/60168330/keys-not-ids-toward-personal-illegibility). No accounts are required.
- **Streamable**: CBOR is inherently streamable, and Blake3 hashes enable streaming cryptographic verification.
- **Any kind of data**: Memos can wrap API responses, file bytes, structured data, or anything else. They also provide a mechanism for adding self-certifying metadata (headers) to any data.

### Non-features

- **P2P**: SZDT is transport-agnostic. It's just a file format.
- **Efficiency**: SZDT prioritizes simplicity over efficiency.

## Real-World Applications

### Decentralizable app data

- Build [relay architectures](https://newsletter.squishy.computer/p/natures-many-attempts-to-evolve-a)
- Make HTTP APIs into trustless endpoints
- [Credible exit](https://newsletter.squishy.computer/p/credible-exit) via signed data exports

### Censorship-resistant data archives

- Scientific data archives
- Verifiable HTTP mirrors
- Signed archive files (save SZDT sequences as `.szdt` files analogous to TAR files)
- CBOR is a simple format and an IETF standard that will be around in 10 or 100 years. Good qualities for archival data.

## Technical Advantages

### Over plain HTTP

- Trust is decoupled from origin so data becomes decentralizable.

### Over existing decentralized solutions

- Single opinionated answer for signing AND content addressing to verify BOTH data's authenticity AND integrity.
- Uses Blake3 throughout, making streaming verification possible at every level.
- Decouples trust from transport, allowing use with any protocol, p2p, HTTP, email, sneakernet, whatever.
- It's "just" CBOR, DIDs, Blake3, and Ed25519. Easy to implement, easy to integrate into existing stacks.

## Implementation Highlights

- Rust library with ergonomic API
- CLI for generating signed file archives
- The beginnings of a [nickname (petname) system](https://newsletter.squishy.computer/p/nickname-petname-system)
- **COMING SOON**: Web and Node bindings via WASM

## Future

SZDT is designed to be permissionlessly extensible. We also plan to explore future expansions to the core format:

- More DID methods (e.g. `did:eth`, `did:webvh`...)
- OCAP inspired by [UCAN](https://github.com/ucan-wg/spec)
- e2ee

## Conclusion

SZDT achieves censorship-resistance without exotic protocols. By combining boring technology with zero-trust cryptographic primitives, we can make data censorship-resistant and decentralizable. If there are many copies, and many ways to find them, then data can survive the way dandelions do—by spreading seeds.

All code and docs for the project are open source and released under the MIT license.
