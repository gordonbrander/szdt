---
layout: index.liquid
title: SZDT Explainer
---

# SZDT Explainer

**TLDR**: signed CBOR for censorship-resistant data.

SZDT is a simple format for decentralizing data. SZDT data is self-certifying, so it can be distributed across any protocol, and cryptographically verified without relying on any centralized authority. It is built on broadly available technology (CBOR, Ed25519, Blake3).

## Problem: websites are single points of failure

On the web, trust is centralized around the server. Web resources are accessed by URLs (Uniform Resource Locators), meaning they belong to a single location. This makes web content centralized, vulnerable to lock-in, [link rot](https://en.wikipedia.org/wiki/Link_rot), and censorship. Websites are single points of failure, and single points of failure fail eventually. The only question is when.

The quick fix would be to distribute data to multiple locations, or even across multiple transports (HTTP, BitTorrent, etc...). This would make data resistant to lock-in, link rot, and censorship. Unfortunately, the web's trust model makes this impossible. Since trust is rooted in the centralized server, we can't trust data we get from other sources. There's no way to know if it has been tampered with.

## Solution: self-certifying data

SZDT solves this by combining two big ideas:

- **Public key cryptography**: use cryptographic signatures to trustlessly prove who created the archive.
- **Content addressing**: use cryptographic hashes to address data and trustlessly prove data integrity.

Together, these two technologies give us everything we need to create self-certifying data. We no longer need to trust the server. We can verify with cryptography.

Because SZDT data is self-certifying, it can be decentralized across multiple untrusted servers, and distributed via any protocol—HTTP, BitTorrent, even sneakernet.

## Content Addressing

In traditional systems, we refer to files by their location:

```
https://example.com/data.zip
/home/user/documents/report.pdf
```

In SZDT, data is referred to by its **[BLAKE3 hash](https://en.wikipedia.org/wiki/BLAKE_(hash_function))**, a unique fingerprint computed from the actual bytes:

```
c8d5e6f7a9b8c7d6e5f4g3h2i1j0k9l8m7n6o5p4q3r2s1t0u9v8w7x6y5z4a3b2c1
```

When we request content by hash, we can verify we have the correct data by recomputing the hash. If the content produces the same hash, we know we got the bytes we asked for.

This means we no longer need to trust the server to serve the right bytes, we can verify the data integrity with cryptography. Now we can decentralize data across multiple untrusted sources. The hash proves we got what we came for.

## Cryptographic Signatures

Public-key cryptography lets us "sign" data and prove that the

SZDT data is signed with a user-controlled key, and published with all of the cryptographic information needed to verify its authenticity and integrity.

## Memos: Signed Metadata Envelopes

SZDT is built around **memos**, CBOR metadata envelopes signed with an Ed25519 cryptographic key.

Memos are conceptually made up of two parts, **headers** and a **body** (the data). This memo format will be familiar if you've worked with HTTP or other internet protocols.

```cbor
{
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

The memo contains:

- a **cryptographic signature**, signing over the **protected** data
- a **DID** resolving to the public key used to sign the message
- a **src** containing the Blake3 hash of the "body part" of the memo

This gives us everything we need to verify the authenticity and integrity of the data.

### Protected Headers

Protected headers are protected by the cryptographic signature when the memo is signed.

Memos can contain open-ended headers, but there are a few headers that have defined semantics, such as:

- `iss`: The issuer's (memo author's) DID (Decentralized Identifier). Essentially their public key.
- `iat`: A timestamp indicating when the memo was created (Unix epoch in seconds)
- `src`: The Blake3 hash of the body content for this memo
- `content-type`: The MIME type for the body content
- `prev`: The Blake3 hash of the previous version of this memo
- ...plus arbitrary other headers

For a full description of headers, see the [SZDT Memos specification]({{site.url}}/specs/memos/).

Note that memos dont't contain the actual content. They point to the content via a content address in the `src` header. Since the signature covers protected headers, and the `src` hash can be used to prove the integrity of the body, the body is also protected by the signature. Signing over a content address in this way allows for streaming verification, as well as other optimizations — but we'll get to that in a bit.

### Unprotected Headers

Unprotected headers can be modified without breaking the signature:

- `sig`: The Ed25519 signature over the protected headers

Currently the only defined unprotected header is `sig`. Most headers should be protected, but unprotected headers are a good place to put signature data.

## SZDT Sequences

Content addressing gives us a lot of freedom over how content is distributed and retreived. Memos and body content can be distributed separately and retrieved via multiple methods, such as content addressed HTTP file servers, or p2p protocols like [Iroh](https://www.iroh.computer/).

However, sometimes we want to distribute memos and bodies together, or even distribute multiple memos together. Enter **SZDT Sequences**.

An SZDT sequence is a standard CBOR Sequence: just multiple CBOR objects concatenated one after the other:

```
memo1 | content1 | memo2 | content2 | memo3 | content3...
```

Sequences can be handy for distributing SZDT data in bulk, in a single response, or in the form of a file.

That's pretty much all there is to it, but you can check out the [SZDT Sequences spec]({{site.url}}/specs/sequences/) for more details.

## Verified streaming, Blake3's superpower

SZDT uses Blake3, a fast and secure cryptographic hash function that enables **verified streaming**. CBOR and CBOR sequences are also streaming-friendly, making SZDT a great way to move large amounts of data efficiently.

Most cryptographic hashing functions require the content to be fully hashed before the hash can be verified. Blake3 uses a clever Merkel Tree structure that allows data to be verified as it is streamed in. If the data is invalid, we can exit early.

Additionally, Blake3 allows us to hash arbitrary slices *within* bytes, and prove that the hash of the slice belongs to the parent content address. This gives us the ability to generate cryptographically verifiable indexes into SZDT data.

For more about Blake3 verified streaming, see [Bao](https://github.com/oconnor663/bao).

## Summing it up

SZDT provides building blocks for censorship-resistant data distribution:

- Decentralizable data: data is self-certifying. It can be distributed across multiple servers and protocols, and requires no central authorities to verify.
- Transport-independence: SZDT is just a data format, making them easy to distribute via various protocols.
- Verifiable streaming: SZDT uses Blake3, allowing us to cryptographically prove integrity as we stream.
- Developer Friendly: the format is built around widely available tech and standards: CBOR, Ed25519 keys, and Blake3.

Information wants to be free. SZDT makes sure it stays that way.
