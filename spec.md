# SZDT

**S**igned **Z**ero-trust **D**a**T**a

Signed .car files for censorship-resistant publishing and archiving. Pronounced "Samizdat".

## Motivation

Web resources are accessed by URLs (Uniform Resource Locators), meaning they belong to a single canonical location, or "origin". Security on the web is also [dependent upon origin](https://developer.mozilla.org/en-US/docs/Web/Security/Same-origin_policy), making [mirroring](https://en.wikipedia.org/wiki/Mirror_site) difficult.

All of this means web content is effectively centralized. Centralization makes web data vulnerable to lock-in, censorship, and more banal forms of failure, like [link rot](https://en.wikipedia.org/wiki/Link_rot). A website is a [Single Point of Failure](https://en.wikipedia.org/wiki/Single_point_of_failure) (SPOF), and single points of failure fail eventually. The only question is when.

These limitations may not be a problem for some kinds of content (private messaging, corporate apps), but they become pressing in the case of archival information and publishing. For example, scientific datasets, journalism, reference information, libraries, academic journals, etc are often intendend to be broadly accessible public goods, available in perpetuity. However, the websites hosting them can and do disappear, as in the 2025 case of [CDC datasets being taken down by the US government](https://www.theatlantic.com/health/archive/2025/01/cdc-dei-scientific-data/681531/). They may also be subject to censorship in many contexts.

This is a silly situation to be in. [The internet was designed to be decentralized](https://newsletter.squishy.computer/p/decentralization-enables-permissionless)—decentralized enough to survive a nuclear war. Yet the web and internet have centralized through an [unfortunate combo of technical choices and commercial pressures](https://newsletter.squishy.computer/i/65395829/redecentralizing-the-web). We can't fix all of that, but we can make it easier to distribute content to multiple redundant locations. Let's de-SPOF data.

To maintain a resilient information ecosystem, we need a simple way to publish and archive information that:

- Is decentralized, redundant, and censorship-resistant
- Keeps long-tail content alive over long periods of time
- Is easy to adopt **right now**, with infrastructure that is already widely deployed.

## The idea

**TLDR**: a cryptographically-signed .car file containing:

- **Files** stored as raw bytes
- **Links** to additional external files, with content addresses for verifying integrity and multiple redundant URLs for retrieval
- **Address book**, mapping known public keys to [petnames](https://files.spritely.institute/papers/petnames.html).
- **Cryptographic claims** proving the authenticity of the archive

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

# Speculative specification

## High‑level goals

| Requirement | Design choice |
|-------------|---------------|
| _Self‑verifying per‑object_ | Every blob is stored as a **raw IPLD block** whose CID’s multihash is the SHA‑256 of the exact bytes in the block. |
| _Self‑verifying per‑archive_ | The archive header contains one or more **JWT claims** whose payload commits to the set a CID in the archive and is signed by a public key that can be resolved independently (e.g. DID). |
| _Streaming friendliness_ | Sticks to CAR v1’s _length‑prefixed block stream_ so writers can pipe without seeking. |
| _Forward compatibility_ | Extra header field `claims` is added; CAR v1 readers that only read `roots`+`version` will simply ignore it. |

## File layout

SZDT is built on top of [DASL CAR v1](https://dasl.ing/car.html), a simple file format that contains CBOR headers, followed by blocks of content-addressed data.

```
|------- Header -------| |------------------- Data -------------------|
[ int | DAG-CBOR block ] [ int | CID | block ] [ int | CID | block ] …
```

Because every block is content-addressed, CAR files contain everything you need to verify the integrity of the archive. CAR is used by Bluesky's [ATProtocol](https://atproto.com/specs/repository), and the IPFS ecosystem, making it a good starting point.

On top of this foundation, SZDT layers cryptographic claims (placed in the CAR header metadata), and a dag-cbor manifest describing files, links, and other data belonging to the SZDT arcvhive.

## Header schema

An SZDT CAR header has the following structure:

```cbor
{
  "version": 1,
  "roots": [ <cid-bytes>, … ],
  "claims": [ <jwt-utf8>, … ]   ; new, optional
}
```

* `roots` MUST contain the dag-cbor CID for the archive manifest.
* `claims` contains array of zero or more JWT claims signing over a CID, which is typically the CID for the archive manifest.

## Content Identifiers (CIDs)

Content proofs are described with Content Identifiers (CIDs). CIDs are essentially file hashes with some additional bytes for metadata and version information. A CID's structure is:

```
<multibase><version><multicodec><multihash><length><digest>
```

...where multibase, version, multicodec, and multihash are [LEB128](https://en.wikipedia.org/wiki/LEB128) integers describing CID encoding, cid version, data format, and hash function respectively. Length is a LEB128 integer describing digest length, and digest is the bytes of the hash digest.

SZDT supports two kinds of CID, specified in [DASL CID](https://dasl.ing/cid.html).

- CID v1 raw SHA-256
- CID v1 dag-cbor SHA-256

CIDs may be encoded as string or bytes. When encoded as string, only lowercase base32 multibase is supported.

## Archive manifest

The archive manifest is a dag-cbor map containing a map of files and contacts.

```typescript
type ArchiveManifest = {
  // Display name for archive
  dn: String;
  // Map of file paths to links
  files: Record<String, Link>;
  // Map of DIDs to petnames
  contacts: Record<Did, String>;
}

type Link = {
  // CID for file
  cid: CID;
  // Zero or more URLs where it may be found
  location: Url[];
}
```

## CAR block ordering

SZDT writers SHOULD write blocks in depth-first, first seen order, when traversing from `roots`. This ensures efficient streaming when reading.

Practically speaking, assuming the archive manifest is the only root, this should mean that the archive manifest block should be written first.

## Claims

- **Serialization**: JWS Compact (`<base64url(header)>.<base64url(payload)>.<sig>`)
- **Algorithm**: Ed25519 (`EdDSA`).
- **Header** MUST include `kid` with `did:key` so verifiers can obtain the public key.

### Payload claims (registered + private)

| Claim | Type | Description |
|-------|------|-------------|
| `iss` | URI  | Issuer (e.g. `did:key:…`). |
| `iat` | Int  | Issued‑at (seconds since epoch). |
| `exp` | Int? | (Optional) Expiry. |
| `cid` | String\[] | lowercase base32 CID string being vouched for. |
| `carDigest` | String | Lowercase hex SHA‑256 of the exact header *without* the `proofs` key (prevents header swaps). |
| `type` | `"car-proof-v1"` | Explicit type tag for extensibility. |

> **Signing input**: the canonical JSON payload bytes (UTF‑8, no whitespace) are signed. Verifiers MUST canonicalise before hashing.

## Verification procedure

1. **Per‑block integrity**
   *Read block → hash → compare to CID multihash.*
2. **Header integrity**
   *Compute SHA‑256 of the header minus `proofs`; compare to `carDigest` in each proof.*
3. **Signature validity**
   *For every JWT in `proofs`:*
   a. Resolve public key from `kid` or `jwk`.
   b. Verify JWS signature.
   c. Check `exp`, `iat` as usual.
4. **CID coverage**
   *The set in `cids` MUST equal (or superset) the set in `roots`.*

If all steps succeed, both every blob and the top‑level collection are authenticated.

---

## Example (illustrative)

```jsonc
// Header (object before CBOR encoding)
{
  "version": 1,
  "roots": [
    "bafkreigh2akiscaildcf…"        // manifest
  ],
  "claims": [
    "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCIsImtpZCI6ImRpZDprZXk6ejZN..."  // compact JWT
  ]
}
```

## Extensibility & interop notes

* **Unknown header keys**: The CAR v1 spec states that only `roots` and `version` are required; decoders that ignore unknown keys remain compliant.
* **Multiple manifests**: To avoid gigantic root arrays, store one DAG‑CBOR manifest node listing all blob CIDs and use *that* CID as the single root.
* **CAR v2 compatibility**: The same header object can be embedded verbatim inside a CAR v2 payload file.

# Appendix

## References

## Packging format

A SZDT archive is a [CBOR](https://cbor.io/) file. CBOR is chosen for its simplicity, ability to efficiently encode binary data, and its increasing ubiquity.

> 📘 Why CBOR? Why not...
> - CBOR
>   - Advantages
>     - IETF standard
>     - Streaming parsing
>     - Widely available libraries
>   - Disadvantages
>     - Not human readable
>     - Higher barrier to entry vs JSON
> - JSON
>   - Advantages
>     - Ubiquitous
>     - Human-readable and authorable
>   - Disadvantages
>     - Can't embed file bytes without encoding
> - Zip
>    - Advantages
>      - Ubiquitous
>      - Users can un-archive using built-in OS tools
>      - Random access
>    - Disadvantages
>      - No streaming parsing
>      - Higher barrier to entry in browser environment

## Example use-cases

Here are a few things you can do, or that should be easy to do with SZDT.

- Publishing and downloading via HTTP.
- Downloading unpacked files (or zipped archive) from the browser.
    - Accomplishable through a JS library or browser extension.
- Sharing via [sneakernet](https://en.wikipedia.org/wiki/Sneakernet) (USB, Airdrop, SD cards, etc). Sneakernet methods become important when the internet is censored or shut down.
  - During the 2019-2020 Hong Kong protests, protestors used Airdrop to route around censorship: "AirDrop was used to broadcast anti-extradition bill information to the public and mainland tourists" ([Wikipedia](https://en.wikipedia.org/wiki/2019%E2%80%932020_Hong_Kong_protests#Moderate_group)).
  - SD cards are small and high-capacity. Most Android phones can load data, as well as side-load apps directly from an SD card placed in the expansion slot.
- Cold storage on hard drives
    - Private data backups
    - Long-term archival

## FAQ

- Why not just use torrents?
    - We do use Torrents! ...and HTTP, and anything else that can move bits.
- Why support HTTP? Why not just a Torrent Tracker/IPFS/etc?
    - Setting up a BitTorrent or IPFS server isn't always easy. Many ISPs block or throttle common p2p ports, and outbound bandwidth costs can be hard to manage.
    - BitTorrent and other p2p protocols often have trouble keeping long-tail archival content alive.
    - The world is optimized around serving files over HTTP. It's easy, it's everywhere, and we should leverage it for redundancy and availability, while additionally leveraging Torrents when needed.
- Why embed files? Why not just use links?
    - Nothing is more decentralized than having the actual bytes. Making it possible to embed file bytes supports redundancy, resiliency, censorship resistance.
- Why links? Why not just embed files?
    - Some archives are too big to fit into one file. Links allow authors to externalize resources that might be difficult or expensive to embed.
    - Having both links and files allows authors to "turn the dial" on where the bytes live. For example, you might embed text files, but link out to LOTR_4K_ExtendedEdition.mp4.
- Why not Internet Archive/Library of Congress/LibGen/Anna's Archive/Sci-Hub?
    - SZDT could be used with or alongside any of these. Our goal is not to replace archival projects, but to offer a (possibly complimentary) "dandelion seed" file format.
    - Like our namesake, the focus of SZDT is on censorship-resistant self-publishing. Our use-cases include archiving, but also smaller-scale tasks such as personal archives, notes, and other content that may never make it into these larger archives.
- Why CBOR? Why not ZIP?
    - CBOR is fast and easy to unpack in a browser context.
    - CBOR is an IETF standard with broad support across languages, and an increasingly capable toolchain.
- Why censorship-resistance?
    - Censorship-restance is another way of saying "resilience". There are many reasons to want resilient, decentralized knowledge repositories.

## Design principles

- Seeding content should be as easy as uploading a file to an ordinary HTTP server.
- Runs in the browser. If it doesn't, you're DOA. Ideally it "just works", but given the pervasiveness of cross-origin restrictions, we'll settle for a JavaScript polyfill.
- Who and what, not where and how. SZDT verifies authenticity (who) with public keys, and file integrity (what) with hashes. We don't care where the file lives (which server) or how you got it (which transport). This enhances censorship resistance, since the data can be shared through any mechanism.
- Think in files, because the world thinks in files. We don't worry about content chunking, like BitTorrent, or content-addressed DAGs, like IPFS. Files are good enough. They aren't perfect, but they are simple and ubiquitous.
- Be fault-tolerant. Embrace the fact that we might not be able to retrieve every part of an archive. Some files might disappear. It's better to get some than none.

## Acknowledgements

SZDT begs, borrows and steals inspiration from a number of smart people and projects:

- [Noosphere](https://github.com/subconsciousnetwork/noosphere): for the idea of combining archives with p2p discovery via address books and petnames.
- [Nostr](https://nostr.com/protocol): for the emphasis on extreme simplicity, self-sovereign signing, and the use of boring HTTP relays for censorship-resistant publishing.
- WebPackaging / WebBundles proposal for sketching out a way to bundle web resources and distribute them across multiple origins.
- BitTorrent: for being a p2p protocol that actually works.
- Magnet links: for the idea of describing redundant locations.
- [Iroh](https://github.com/n0-computer/iroh), for showing that the ideal chunk size is large, not small. This obliquely inspired us to say that files are good enough. They're a chunk that is already compatible with existing infrastructure.
- RSS, for being super easy to adopt using existing infrastructure.

Other related projects and prior art:

- Hashlink
- did:key
- Metalink: offering a similar idea, and for helping define some some non-goals, such as content-chunking. We can fall back to BitTorrent for effecient chunked downloads. Our design goal is resilience, not efficiency.
