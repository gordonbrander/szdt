# SZDT

**S**igned **Z**ero-trust **D**a**T**a. Pronounced "Samizdat".

Signed CBOR for censorship-resistant data archives.

## Intro

**TLDR**: a cryptographically-signed [CBOR sequence](https://www.rfc-editor.org/rfc/rfc8742.html) containing metadata, bytes, and everything needed to verify them.

- **Zero trust**: Archives are cryptographically signed and content-addressed using Blake3 hashes, requiring no trusted authorities for verification.
- **Verified streaming**: Blake3 and Bao enable streaming verification of archive integrity without buffering entire files.
- **Verified random access**: Optional manifests provide efficient seeking to specific content via HTTP range requests or file seeks.

SZDT makes use of the Blake3 hashing algorithm to enable efficient streaming and random access while cryptographically verifying data integrity.

## Motivation

Web resources are accessed by URLs (Uniform Resource Locators), meaning they belong to a single canonical location, or "origin". Security on the web is also [dependent upon origin](https://developer.mozilla.org/en-US/docs/Web/Security/Same-origin_policy), making [mirroring](https://en.wikipedia.org/wiki/Mirror_site) difficult.

All of this means web content is effectively centralized. Centralization makes web data vulnerable to lock-in, censorship, and more banal forms of failure, like [link rot](https://en.wikipedia.org/wiki/Link_rot). A website is a [Single Point of Failure](https://en.wikipedia.org/wiki/Single_point_of_failure) (SPOF), and single points of failure fail eventually. The only question is when.

These limitations may not be a problem for some kinds of content (private messaging, corporate apps), but they become pressing in the case of archival information and publishing. For example, scientific datasets, journalism, reference information, libraries, academic journals, etc are often intendend to be broadly accessible public goods, available in perpetuity. However, the websites hosting them can and do disappear, as in the 2025 case of [CDC datasets being taken down by the US government](https://www.theatlantic.com/health/archive/2025/01/cdc-dei-scientific-data/681531/). They may also be subject to censorship in many contexts.

This is a silly situation to be in. [The internet was designed to be decentralized](https://newsletter.squishy.computer/p/decentralization-enables-permissionless)—decentralized enough to survive a nuclear war. Yet the web and internet have centralized through an [unfortunate combo of technical choices and commercial pressures](https://newsletter.squishy.computer/i/65395829/redecentralizing-the-web). We can't fix all of that, but we can make it easier to distribute content to multiple redundant locations. Let's de-SPOF data.

To maintain a resilient information ecosystem, we need a simple way to publish and archive information that:

- Is decentralized, redundant, and censorship-resistant
- Keeps long-tail content alive over long periods of time
- Is easy to adopt **right now**, with infrastructure that is already widely deployed.

## Goals

- **Zero-trust**: SZDT archives are verified using cryptographic hashing and public key cryptography. No centralized authorities are required.
- **Decentralized**: [Lots Of Copies Keeps Stuff Safe](https://www.lockss.org/). SZDT archives are made to be distributed to many redundant locations, including multiple HTTP servers, BitTorrent, hard drives, etc. Likewise, URLs in SZDT files point to many redundant locations, including HTTP servers, BitTorrent, and more.
- **Censorship-resistant**: Distributable via HTTP, Torrents, email, airdrop, sneakernet, or anything else.
- **Anonymous/pseudonymous**: SZDT uses [keys, not IDs](https://newsletter.squishy.computer/i/60168330/keys-not-ids-toward-personal-illegibility). No accounts are required. This allows for anonymous and pseudonymous publishing. If an author wishes to be publicly known, they can use other protocols to link a key to their identity.
- **Discoverable**: SZDT archives contain multiple pointers to places where other archives and keys can be found. With just one or two SZDT files, you can follow the links to construct your own web of trust and collection of archives.
- **Boring**: SZDT uses ubiquitous technology. It is designed to be compatible with widely deployed infrastructure, like HTTP. The format is simple, and easy to implement in any language.

If there are many copies, and many ways to find them, then data can survive the way dandelions do—by spreading seeds.

### Non-goals

- **P2P**: SZDT is transport-agnostic. It's just a file format. You should be able to publish, share, and retreive SZDT archives from anywhere, including HTTP, BitTorrent, email, messaging apps, sneakernet, etc.
- **Efficiency**: SZDT is not efficient. Its goal is to be simple and resilient, like a cockroach. We don't worry about efficient chunking, or deduping. When efficient downloading is needed, we leverage established protocols like BitTorrent.
- **Comprehensive preservation**: SZDT doesn't aim for comprehensive preservation. Instead it aims to make it easy to spread data like dandelion seeds. Dandelions are difficult to weed out.

## Design

SZDT archives are CBOR sequences of **memos**, each containing open-ended headers and data. Everything is represented as a memo, and archives can contain any number of memos.

### Memos

Memos are CBOR arrays of `[headers, body]`

- **headers**: CBOR map containing metadata. Like HTTP headers, memo headers can contain open-ended metadata. Any number of header fields are allowed, some fields, like `content-type`, are "blessed" by the spec.
- **body**: A CBOR value representing the body part of the memo. The type of the body is determined by the `content-type` header. It is typically bytes.

Example memo:

```
[
  {                               // headers (CBOR map)
    "content_type": "text/plain", // mime type of body
    "path": "/example.txt",
    "did": "did:key:...",         // DID for signature verification
    "sig": h'...',                // cryptographic signature (optional)
    "digest": h'...'              // Blake3 hash of body
    // ...other metadata
  },
  h'...'       // bytes (raw content)
]
```

Memos can be cryptographically signed. Signed memos carry all of the information needed to trustlessly verify the authenticity and integrity of the memo.

### Archives

Archives are CBOR sequences - a concatenation of individual CBOR-encoded memos:

```
archive = memo1 + memo2 + memo3 + ...
```

All CBOR items use the deterministic [CBOR/c ("CBOR Core")](https://datatracker-ietf-org.lucaspardue.com/doc/draft-rundgren-cbor-core/) profile so that items can be deterministically hashed, signed, and content-addressed.

### Manifests

Optional manifests provide efficient random access:

```
manifest_memo = [
  {
    "content-type": "application/vnd.szdt.manifest+cbor",
    "digest": h'...'              // Blake3 hash of body
    // ...other metadata
  },
  {
    entries: manifest_entry[]
  }
]

manifest_entry = {
  "path": "/path/to/file",     // file path
  "length": 1234,              // memo length in bytes
  "digest": h`blake3_hash`     // Blake3 hash of memo
}
```

Manifests can be distributed:
- **Embedded**: As the first memo in the archive
- **Sidecar**: Generated or distributed independently

With a manifest, archives support efficient random access through:

- **HTTP range requests**: `Range: bytes=offset-end`
- **File seeking**: Direct file system seeks

## Development

### Installing binaries on your path with Cargo

From the project directory:

```bash
cargo install --path .
```

This will install the binaries to `~/.cargo/bin`, which is usually added to your path by the Rust installer.
