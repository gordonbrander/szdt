# SZDT

**S**igned **Z**ero-trust **D**a**T**a

A simple format for distributed, censorship-resistant publishing and archiving. Pronounced "Samizdat".

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

**TLDR**: a self-verifiying archive file with...

- **Cryptographic signatures** proving authenticity independent of origin
- **Cryptographic hashes** proving integrity independent of origin
- **Files** encoded as bytes
- **Links** to additional external files, with redundant URLs and/or magnet links for retrieval
- **Address book**, mapping known public keys to [petnames](https://files.spritely.institute/papers/petnames.html).

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

SZDT is a self-verifying format. A SZDT file comes with an attached set of zero-trust cryptographic proofs giving you everything you need to verify the archive's authenticity and integrity, regardless of where it came from.

SZDT is built on top of widely deployed formats (HTTP headers, JWTs), making it easy to adopt into existing archival workflows.

The format is made up of two independent specifications:

- An HTTP-style header envelope, containing metadata and proofs
- An archive manifest file, encoded as CBOR

Each of these formats may be used independently. The envelope format may be used encode metadata and proofs for arbitrary file types, such as TAR or ZIP.

### Header envelope

SZDT metadata is encoded in the same way as HTTP headers. That is, a series of key-value pairs, separated by a colon (":"), and terminated by a CLRF (`\r\n`).

An abbreviated ABNF definition is provided below. For the full ABNF definition, consult [RFC 9112](https://datatracker.ietf.org/doc/html/rfc9112).

```abnf
HTTP-message  = start-line CRLF
                  *( field-line CRLF )
                  CRLF
                  [ message-body ]

field-line = field-name ":" OWS field-value OWS
```

Each field line consists of a case-insensitive field name followed by a colon (":"), optional leading whitespace, the field line value, and optional trailing whitespace.

Headers are terminated by a final CRLF (`\r\n`) (displayed in text as a blank line).

Example:

```
szdt/1.0
content-type: "text/plain"

Body text
```

Just as in HTTP, the message body may contain arbitrary data. This means an SZDT envelope may be used to wrap any other file type.

#### Reserved headers

Header field names that are reserved in HTTP should be considered reserved in SZDT, and to have the same semantics as in HTTP.

In addition, SZDT defines a set of reserved header fields.

- `dn`: "Display Name". A hint suggesting a filename for the underlying file. Value must be a valid file path and must not contain any path traversal characters. Consumers may use this hint in any way they see fit.
- `szdt-cid`: A content identifier (CID) that content-addresses the body part of the SZDT envelope. The CID may be used as an integrity check for the body, and as a target for signatures making claims about the body. CIDs in this header must follow the format outlined in the CID section of this specification.
- `szdt-claim`: A [JSON Web Token](https://datatracker.ietf.org/doc/html/rfc7519) (JWT) making a cryptographic claim. SZDT uses claims to make authorship and other assertions about the body. Multiple claims may be made by including multiple `szdt-claim` headers or by using multiple header syntax (comma-separated values).

Other than that, producers are free to define custom headers describing arbitrary metadata. Any number of fields may be included in the headers.

#### Wrapping and unwrapping file data

SZDT tooling should provide tools for "wrapping" and "unwrapping" file data of any type.

When wrapping file bytes in an SZDT envelope, tools should encode a `content-type` header describing the [Media Type](https://www.iana.org/assignments/media-types/media-types.xhtml) of the original file. Tools may also include a `dn` header describing a hint for the filename of the original file.

When unwrapping a file, tools may use `content-type` or `dn` headers as hints to determine an extension for the the unwrapped file.

### CIDS (Content Identifiers)

https://dasl.ing/cid.html

### Cryptographic assertions

SZDT claims and proofs are encoded as [JSON Web Tokens](https://datatracker.ietf.org/doc/html/rfc7519) (JWTs), placed in a header field of the SZDT envelope. JWTs are broadly adopted, making SZDT claims easy to adopt in existing workflows.

```
szdt-claim: <JWT>
```

Any number of claims can be attached to an archive (described as multiple headers or comma-separated in a single header).

Trust is established through the cryptographic proofs provided in these claims. The headers in the envelope are themselves unsigned. This allows any number of authors to contribute additional assertions without invalidating previous proofs.

### Content assertion

The most common assertion is a content assertion, which proves the integrity of the data to which it is attached.

Content proof is based on the signed SHA-256 hash of the archive contents.

Example:

```json
{
  "knd": "szdt/ast/content", // Assertion kind
  "alg": "EdDSA", // Algorithm used to sign
  "iss": "did:key:z6Mk...", // DID key of the entity that is issuing the assertion
  "hash": {
    "alg": "sha256", // Hash algorithm used to compute the digest
    "digest": "..." // Base64url-encoded digest
  },
  "meta": {}
}
```

The following JWT fields are required in content assertions:

- `alg`: The cryptographic algorithm used to sign the assertion. Only `EdDSA` is supported at this time.
- `iss`: The DID of the entity that issued the assertion. Only `did:key` is supported at this time.
- `hash.alg`: The hash algorithm used to compute the digest. Only `sha256` is supported at this time.
- `hash.digest`: The base64url-encoded digest of the archive contents.
- `meta`: Additional arbitrary metadata about the assertion.

Other valid JWT fields, such as `aud` may be used to provide additional information about the assertion, but are not required.

It is recommended that `exp` and `nbf` fields be included to specify the expiry and not-before times of the assertion, respectively.

#### Content hashing process

> TODO: define canonical hashing process over file collections that is generic to content type.
>
> Needs to have:
> - File path
> - File bytes
> - Other file metadata such as modified and ACLs?
>   - Check what TAR supports
> - Streamable (ideally hash as we go), rather than two-pass
>
> Options:
> - Canonically encode as TAR and sign
> - Canonically encode as CBOR and sign

### Updates assertion

Updates assertions provide a list of URLs that can be checked for updates to the content of the archive. Archives are logically identified by their `iss` and `id` fields, where `iss` is the DID key of the agent that issued the assertion and `id` is a unique identifier for the archive for that agent.

Example:

```json
{
  "knd": "szdt/ast/updates", // Claim kind
  "alg": "EdDSA", // Algorithm used to sign
  "iss": "did:key:z6Mk...", // DID key of the agent that issued the assertion
  "iat": 1630456800, // Issued at time (UNIX timestamp in seconds)
  "id": "urn:uuid:123e4567-e89b-12d3-a456-426614174000", // Unique identifier for the archive
  "urls": [
    "https://example.com/archive.tar",
    "https://example2.com/archive.tar"
  ],
  "meta": {}
}
```

The following JWT fields are required in content assertions:

- `iss`: DID key of the agent that issued the assertion.
- `iat`: Issued at time (UNIX timestamp in seconds). Used to determine the order of updates.
- `id`: Unique identifier for the archive. Must be a valid URI. A UUID URN is often used.
- `urls`: List of URLs that can be checked for updates to the content of the archive.
- `meta`: Arbitrary key-value metadata.

#### Archive update sematics

Update assertions define an update history **according to the key used to sign the assertion**. Since there may be multiple update assertions signed by different keys, a single archive file may describe multiple possible update lineages. It is up to the consumer to determine which issuing agent (`iss`), and therefore which update history, they are interested in following.

Beginning with the update assertions from the `iss` agent you are interested in,

- Select the newest update assertion by comparing the `iat` field of each valid assertion from the desired `iss`.
- Retrieve the content of the archive from any of the URLs specified in the `urls` field of the assertion.
  - If the archive is not found, try another URL in the list.
  - If all URLs fail, stop.
- Upon successfully retrieving an archive, check for an update assertion from the same `iss`.
  - If an update assertion from the same key is not found, stop.
  - If no valid update assertions from the same key are found, stop.
  - If one or more update assertions from the same key are found, select the newest one by comparing the `iat` field of each valid assertion from the same key.
- Verify the signature of the newest valid update assertion using the public key associated with the `iss`.
  - If the signature is invalid, stop.
- The assertion is considered to be the newest revision of the archive.

It is up to clients how to handle updates. For example, a client designed to checks out the most recent version of an archive may replace an old versions with the new one, whereas an archival client may choose to keep all revisions.

### Cryptosuite



### Attaching assertions to archives

The outer COSE_Sign1 envelope is described in [RFC 9052](https://www.rfc-editor.org/rfc/rfc9052.html#name-signing-with-one-signer). It has the following high-level structure:

```typescript
// TODO redo this in CBOR Diagnostic Language.
type COSE_Sign1 = [
    Uint8Array, // Protected headers
    Record<string | number, unknown>, // Unprotected headers
    Uint8Array, // Body bytes
    Uint8Array, // Ed25519 signature bytes
];
```

The COSE_Sign1 envelope used for SZDT should be tagged with [CBOR tag 18, the tag for `COSE_Sign1`](https://www.rfc-editor.org/rfc/rfc9052.html#section-4.2-2). SZDT authoring programs must tag the envelope with this tag. SZDT clients should attempt to read the COSE_Sign1 envelope, even if the envelope is not tagged.

The envelope should include specific header data in the protected headers:

- `kid`: a DID which resolves to a public key that can be used to verify the signature. Only the [`did:key`](https://github.com/digitalbazaar/did-method-key) is expected to be supported at this time. The `did:key` must encode an `Ed25519` public key.
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
    meta?: Record<string | number, unknown>; // Arbtrary metadata
}

type Link = {
    type: "Link";
    path: string;
    urls: string[]; // HTTP, magnet, etc
    filehash: Uint8Array; // multihash SHA-256 of file bytes
    meta?: Record<string | number, unknown>; // Arbtrary metadata
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

`meta` fields are provided in many structures of the archive to support workflows where catalog metadata, such as ISBNs, are stored alongside the file bytes and URLs. We expect some archives may consist entirely of catalog metadata and URLs. Distributing metadata can, in many cases, be as valuable as distributing the archival data itself.

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
