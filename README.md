# SZDT

**S**igned **Z**ero-trust **D**a**T**a. Pronounced "Samizdat".

Signed CBOR for censorship-resistant data.

## Intro

**TLDR**: a cryptographically-signed [CBOR sequence](https://www.rfc-editor.org/rfc/rfc8742.html) containing metadata, bytes, and everything needed to verify them.

- **Zero trust**: Archives are cryptographically signed and content-addressed using Blake3 hashes, requiring no trusted authorities for verification.
- **Verified streaming**: Blake3 and Bao enable streaming verification of archive integrity without buffering entire files.
- **Verified random access**: Optional manifests provide efficient seeking to specific content via HTTP range requests or file seeks.

SZDT makes use of the Blake3 hashing algorithm to enable efficient streaming and random access while cryptographically verifying data integrity.

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

### Setup

Requirements:

- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/en/download/)

Once you have Rust and Node, you can get the additional dependencies by running:

```bash
./scripts/setup.sh
```

### Installing CLI on your path with Cargo

From the project directory:

```bash
cargo install --path rust/
```

This will install the binaries to `~/.cargo/bin`, which is usually added to your path by the Rust installer.
