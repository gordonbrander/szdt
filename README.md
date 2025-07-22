# SZDT

**S**igned **Z**ero-trust **D**a**T**a. Pronounced "Samizdat".

Signed CBOR for censorship-resistant data.

## Intro

**TLDR**: cryptographically-signed CBOR envelopes containing data, metadata, and everything needed to trustlessly verify it.

- [Whitepaper](./WHITEPAPER.md)
- [Specs and docs](https://gordonbrander.github.io/szdt/)

## Features

- **Zero-trust**: SZDT archives are verified using cryptographic hashing and public key cryptography. No centralized authorities are required.
- **Censorship-resistant**: Because trust is decoupled from origin or transport, SZDT archives can be distributed via HTTP, Torrents, email, airdrop, sneakernet, or anything else that is available.
- **Decentralizable**: SZDT decouples trust from origin, so data can be distributed to many redundant locations, including multiple HTTP servers, BitTorrent, hard drives, etc. [Lots Of Copies Keeps Stuff Safe](https://www.lockss.org/).
- **Anonymous/pseudonymous**: SZDT uses [keys, not IDs](https://newsletter.squishy.computer/i/60168330/keys-not-ids-toward-personal-illegibility). No accounts are required.
- **Streamable**: CBOR is inherently streamable, and Blake3 hashes enable streaming cryptographic verification.
- **Any kind of data**: Memos can wrap API responses, file bytes, structured data, or anything else. They also provide a mechanism for adding self-certifying metadata (headers) to any data.

### Non-features

- **P2P**: SZDT is transport-agnostic. It's just a file format.
- **Efficiency**: SZDT prioritizes simplicity over efficiency.

## Development

### Prerequisites

- [Node](https://nodejs.org/en/download) v24 or later
- [Rust](https://www.rust-lang.org/) v1.88 or later

### Setting up dev environment

- Clone the repository
- Run `./scripts/setup.sh` to install development dependencies (`wasm-pack` and `just`)

Run `just default` to see a list of build commands.

### Building WASM

```bash
just build_szdt_web
```

### Installing CLI from your path

From the project directory:

```bash
cargo install --path ./rust/szdt-cli
```

This will install the `szdt` binary to `~/.cargo/bin` (which should have been added to your path by the Rust installer).
