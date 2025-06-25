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
- **Decentralized**: [Lots Of Copies Keeps Stuff Safe](https://www.lockss.org/). SZDT archives are made to be distributed to many redundant locations, including multiple HTTP servers, BitTorrent, hard drives, etc.
- **Censorship-resistant**: Distributable via HTTP, Torrents, email, airdrop, sneakernet, or anything else.
- **Anonymous/pseudonymous**: SZDT uses [keys, not IDs](https://newsletter.squishy.computer/i/60168330/keys-not-ids-toward-personal-illegibility). No accounts are required.
- **Resilient**: built on CBOR sequences, an [IETF standard](https://cbor.io/spec.html) that is widely supported and should be readable 10, 20, 100 years into the future.
- **Streamable**: CBOR is inherently streamable, and Blake3 hashes enable streaming cryptographic verification.

### Non-goals

- **P2P**: SZDT is transport-agnostic. It's just a file format.
- **Efficiency**: SZDT prioritizes simplicity and resilience over efficiency.
- **Comprehensive preservation**: SZDT aims to make it easy to spread data like dandelion seeds.

## Specification

See [spec.md](./spec.md).

## Development

### Installing binaries on your path with Cargo

From the project directory:

```bash
cargo install --path .
```

This will install the binaries to `~/.cargo/bin`, which is usually added to your path by the Rust installer.
