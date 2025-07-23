---
layout: index.liquid
title: SZDT - Signed Zero-trust DaTa
---

<img src="{{ "static/logo2.png" | prepend: site.url }}" height="80" />

# SZDT

**S**igned **Z**ero-trust **D**a**T**a. Pronounced "Samizdat".

It's time to make the web censorship-resistant. SZDT decouples trust from servers so that data can spread like dandelion seeds.

<div class="hstack gap">
    <a class="btn primary" href="https://github.com/gordonbrander/szdt/blob/main/WHITEPAPER.md">Whitepaper</a>
    <a class="btn" href="{{ "guides/quickstart/" | prepend: site.url }}">Quickstart</a>
    <a class="btn" href="https://github.com/gordonbrander/szdt">GitHub</a>
</div>

SZDT is signed CBOR with everything needed to cryptographically verify authenticity and integrity. Data can be seeded across cheap commodity HTTP servers or over p2p protocols like [BitTorrent](https://transmissionbt.com/) and [Iroh](https://www.iroh.computer/).

## Supporters

<div class="hstack gap">
    <a rel="external" href="https://cosmos-institute.org/">
        <img class="img"  alt="Cosmos Institute" src="{{ "media/cosmos-institute.svg" | prepend: site.url }}" width="200" height="50" />
    </a>
</div>

## Specifications

SZDT is a set of building blocks for publishing censorship-resistant data.

- [SZDT Whitepaper](https://github.com/gordonbrander/szdt/blob/main/WHITEPAPER.md)
- [SZDT Explainer]({{site.url}}specs/explainer/): Overview of SZDT concepts. Start here.
- [SZDT Memos Specification]({{site.url}}specs/memos/): CBOR metadata envelopes that can be cryptographically signed to create self-certifying data.
- [SZDT Sequence Specification]({{site.url}}specs/sequences/): Bundle multiple CBOR objects into a single content-addressed CBOR Sequence.
- [SZDT Archives Specification]({{site.url}}specs/archives/): a file archiving format built on memos and sequences. Archives provide a censorship-resistant format for distributing collections of signed, verifiable files.

## Features

- **Zero-trust**: SZDT archives are verified using cryptographic hashing and public key cryptography. No centralized authorities are required.
- **Censorship-resistant**: Because trust is decoupled from origin or transport, SZDT archives can be distributed via HTTP, Torrents, email, airdrop, sneakernet, or anything else that is available.
- **Decentralizable**: SZDT decouples trust from origin, so data can be distributed to many redundant locations, including multiple HTTP servers, BitTorrent, hard drives, etc. [Lots Of Copies Keeps Stuff Safe](https://www.lockss.org/).
- **Anonymous/pseudonymous**: SZDT uses [keys, not IDs](https://newsletter.squishy.computer/i/60168330/keys-not-ids-toward-personal-illegibility). No accounts are required.
- **Streamable**: CBOR is inherently streamable, and Blake3 hashes enable streaming cryptographic verification.
- **Any kind of data**: Memos can wrap API responses, file bytes, structured data, or anything else. They also provide a mechanism for adding self-certifying metadata (headers) to any data.
