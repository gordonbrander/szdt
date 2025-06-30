---
layout: index.liquid
title: SZDT - Signed Zero-trust DaTa
---

<img src="/assets/logo2.png" height="80" />

# SZDT

**S**igned **Z**ero-trust **D**a**T**a. Pronounced "Samizdat".

Signed CBOR for censorship-resistant data.

<div class="hstack gap">
    <a class="btn primary" href="{{ "download/" | prepend: site.url }}">Download</a>
    <a class="btn" href="{{ "guides/quick-start/" | prepend: site.url }}">Quick start</a>
    <a class="btn" href="https://github.com/gordonbrander/szdt">GitHub</a>
</div>

## Supporters

<div class="hstack gap">
    <a rel="external" href="https://cosmos-institute.org/">
        <img class="img"  alt="Cosmos Institute" src="/media/cosmos-institute.svg" width="200" height="50" />
    </a>
</div>

## Specifications

SZDT is a set of building blocks for publishing censorship-resistant data.

- [SZDT Explainer]({{site.url}}specs/explainer/): Overview of SZDT concepts. Start here.
- [SZDT Memos Specification]({{site.url}}specs/memos/): CBOR metadata envelopes that can be cryptographically signed to create self-certifying data.
- [SZDT Sequence Specification]({{site.url}}specs/sequences/): Bundle multiple CBOR objects into a single content-addressed CBOR Sequence.
- [SZDT Archives Specification]({{site.url}}specs/archives/): a file archiving format built on memos and sequences. Archives provide a censorship-resistant format for distributing collections of signed, verifiable files.

## Goals

- **Zero-trust**: SZDT archives are verified using cryptographic hashing and public key cryptography. No centralized authorities are required.
- **Decentralized**: [Lots Of Copies Keeps Stuff Safe](https://www.lockss.org/). SZDT archives are made to be distributed to many redundant locations, including multiple HTTP servers, BitTorrent, hard drives, etc.
- **Censorship-resistant**: Distributable via HTTP, Torrents, email, airdrop, sneakernet, or anything else.
- **Anonymous/pseudonymous**: SZDT uses [keys, not IDs](https://newsletter.squishy.computer/i/60168330/keys-not-ids-toward-personal-illegibility). No accounts are required. This allows for anonymous and pseudonymous publishing. If an author wishes to be publicly known, they can use other protocols to link a key to their identity.
- **Boring**: SZDT uses ubiquitous technology. It is designed to be compatible with widely deployed infrastructure, like HTTP. The format is simple, and easy to implement in any language.

If there are many copies, and many ways to find them, then data can survive the way dandelions do—by spreading seeds.
