# SZDT

**S**igned **Z**ero-trust **D**a**T**a

A container format for distributed, censorship-resistant publishing and archiving. Pronounced "Samizdat".

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

**TLDR**: a cryptographically-signed CBOR object containing:

- **Files** stored as raw bytes
- **Links** to additional external files, with redundant URLs and/or magnet links for retrieval, plus checksums for veryifying file integrity.
- **Address book**, mapping known public keys to [petnames](https://files.spritely.institute/papers/petnames.html).
- **Cryptographic signature** proving the authenticity of the archive

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
- **Efficiency**: SZDT is not efficient. Its goal is to be simple and resilient, like a cockroach. We don't worry too much about efficient streaming or chunking. When efficient downloading is needed, we leverage established protocols like BitTorrent.
- **Comprehensive preservation**: SZDT doesn't aim for comprehensive preservation. Instead it aims to make it easy to spread data like dandelion seeds. Dandelions are difficult to weed out.

## Speculative specification

**TLDR**: SZDT is a cryptographically-signed CBOR object containing binary file data, plus redundant links to additional data.

The format is made up of two part

- an outer signed CBOR COSE_Sign1 envelope, proving the authenticity of the data
- an inner CBOR object describing the archive data

The outer envelope has the following high-level structure:

```typescript
// TODO redo this in CBOR Diagnostic Language.
type COSE_Sign1 = [
    Uint8Array, // Protected headers
    Record<string | number, unknown>, // Unprotected headers
    Uint8Array, // Body bytes
    Uint8Array, // Ed25519 signature bytes
];
```

SZDT archives are expected to include specific header data in the protected headers.

- `kid`: a DID which resolves to a public key that can be used to verify the signature. Only [`did:key`](https://github.com/digitalbazaar/did-method-key) is expected to be supported at this time.
- `cty`: a content type of `application/vnd.szdt.archive+cbor`

```typescript
type ProtectedHeaders = Record<string | number, unknown> & {
  kid: string, // did:key corresponding to signature
  cty: "application/vnd.szdt.archive+cbor", // Content type
},
```

The body bytes contain a CBOR encoded `Archive` object, with the following high-level structure:

```typescript
// Decoded from cbor bytes in memo body
// for cty "application/vnd.szdt.archive+cbor"
type Archive = {
    name: string; // identifier that is unique to the public key
    timestamp: number; // Timestamp denoting when this archive was created.
    files: Array<File | Link>;
    contacts: Contact[];
    urls: []; // URLS where updates for this archive may be found
}

type File = {
    type: "File";
    path: string;
    content: Uint8Array; // File bytes
    meta: Record<string | number, unknown>; // Arbtrary metadata
}

type Link = {
    type: "Link";
    path: string;
    urls: string[]; // HTTP, magnet, etc
    filehash: Uint8Array; // multihash SHA-256 of file bytes
    meta: Record<string | number, unknown>; // Arbtrary metadata
}

type Contact = {
    nickname: string; // Suggested name for key
    pubkey: string; // Ed25519 public key
}
```

- `timestamp`: a timestamp denoting when this archive was created.
- `name`: a name for the archive. The name may be used to logically identify the archive. For example, a client may use the name as a directory name when unpacking archive contents.
- `files`: an array of `File` or `Link` structures, represented as CBOR maps.
  - `File`
    - `path`: a suggested file path when unpacking the archive
    - `content`: the raw bytes of the original file contents
    - `meta`: a CBOR map of arbitrary metadata. Archive authors may use this field to record additional comments and library metadata such as ISBNs, etc.
  - `Link`
    - `path`: a suggested file path when unpacking the archive
    - `urls`: an array of URLs where future updates to the archive may be found. URLs typically include multiple HTTP URLs, but may include any valid URL type, including [magnet links](https://en.wikipedia.org/wiki/Magnet_URI_scheme) to torrents, etc.
    - `filehash`: the [multihash](https://github.com/multiformats/multihash) SHA2-256 hash of the raw file bytes.
    - `meta`: a CBOR map of arbitrary metadata. Archive authors may use this field to record additional comments and library metadata such as ISBNs, etc.
- `urls`: an array of URLs where future updates to the archive may be found. URLs typically include multiple HTTP URLs, but may include any valid URL type, including [magnet links](https://en.wikipedia.org/wiki/Magnet_URI_scheme) to torrents, etc.
- `contacts`: an array of zero or more `Contact` structures, represented as CBOR maps. Can be used by clients to build up their own address book of contacts.
  - `Contact` contains
    - `pubkey`: An Ed25519 public key, acting as the identifier for the contact (multicodec encoded)
    - `nickname`: The [petname](https://files.spritely.institute/papers/petnames.html) given to this public key by the archive. This field may be used as a suggestion by clients, if users decide to add the public key to their own list of contacts.

Meta fields are provided in many fields of the archive to support workflows where catalog metadata, such as ISBNs, are stored alongside the actual file bytes and URLs. We expect some archives may consist entirely of catalog metadata and URLs. Distributing metadata can, in many cases, be as valuable as distributing the archival data itself.

### Signing archives

Signing the memo is accomplished by signing the body bytes with the [COSE_Sign1 signing process](https://www.rfc-editor.org/rfc/rfc9052.html#name-signing-with-one-signer) using the [Ed25519 signature scheme](https://www.rfc-editor.org/rfc/rfc9052.html).

> TODO: consider UCAN, ZCAP, or similar for delegation/key rotation (see Noosphere's approach)

> 📘 Why Ed25519? Why not…
> - Ed25519
>   - Advantages
>   - Disadvantages
> - secp256k1
>   - Advantages
>   - Disadvantages

### Generating file hashes

Cryptographic hashes are used to verify file integrity when linking to resources.

File hashes are generated with the SHA-256 hashing function. SHA-256 is chosen for its ubiquity and cryptographic security.

The `filehash` is the SHA-256 hash in [multihash](https://github.com/multiformats/multihash) format. In practice, this means it will have a two byte prefix of `0x12 0x20`.

> 📘 Why Sha256? Why not...
> - Sha256
>   - Advantages
>     - Ubiquitous
>     - Used by Nostr for hashes
>   - Disadvantages
>     - Iroh doesn't use it
> - Blake3
>   - Advantages
>     - Small and fast hashes
>     - [Iroh](https://iroh.computer/) can used them as a "CID" retreive file data over an IPFS-like p2p network.
>     - Streaming verification
>   - Disadvantages
>     - Newer, not ubiquitous
> - MD5
>   - Advantages
>     - Used by Anna's Archive, LibGen
>   - Disadvantages
>     - Hash collisions, insecure. I think this rules it out.

### Updating archives

Clients may choose to treat archives as versioned resources, where the pubkey is the logical identifier for the archive, and updates use a Last-Write-Wins strategy, using the archive timestamp.

> **Note**: Since the archive is signed, we can trust that the timestamp is expressing the author's intention regarding which version of the archive should be considered most recent.

Notably, authors are free to embed structures like CRDTs in the file portions of the format, allowing for even finer-grained merge strategies, although this is outside the scope of this specification, and up to the author.

# Appendix

### Packging format

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
