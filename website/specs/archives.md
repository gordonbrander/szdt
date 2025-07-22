---
layout: index.liquid
title: SZDT Archives Specification
---

# SZDT Archives Specification

SZDT Archives provide a standardized format for creating censorship-resistant signed file archives. SZDT archives can be distributed across multiple channels and verified without trusted authorities. Archives are [SZDT Sequences]({{ "specs/sequences/" | prepend: site.url }}) of [memos]({{ "specs/memos/" | prepend: site.url }}) and file bytes.

## Motivation

Traditional archives (ZIP, TAR, etc.) provide no built-in authenticity verification.

SZDT Archives address these limitations by providing:

- **Cryptographic verification**: Every archive is signed and can be verified without trusted authorities
- **Decentralized distribution**: Archives can be shared via HTTP, BitTorrent, email, sneakernet, or any other transport
- **Censorship resistance**: No single point of failure or control

Archives may be particularly valuable for:
- Scientific datasets that need long-term preservation
- Documentation that may face censorship
- Any collection of files that needs distributed, verifiable storage

## Archive Format

SZDT Archives are [CBOR sequences](https://www.rfc-editor.org/rfc/rfc8742.html) conforming to the [SZDT sequence]({{ "specs/sequences/" | prepend: site.url }}) spec, and having the following high-level structure:

```
archive = memo1 | bytes1 | memo2 | bytes2 | ...
```

Where:
- **memo**: A signed [memo]({{ "specs/memos/" | prepend: site.url }}) pointing to the manifest
- **manifest**: A CBOR-encoded manifest listing all resources in the archive
- **bytes**: CBOR byte strings, representing the raw bytes of each resource, in the order listed in the manifest

## Archive memos

Archive entries are described by a standard [SZDT memo]({{ "specs/memos/" | prepend: site.url }}) that wraps the archive manifest. In addition to the required memo headers, archives add the following protected headers:

- `path`: a hint indicating an appropriate file path when unpacking the archive. The client may choose to interpret this hint in whatever way is appropriate to the context.

### Path Requirements

- Paths MUST start with "/" (absolute within archive)
- Paths MUST use "/" as separator (Unix-style)
- Paths MUST NOT contain ".." components
- Paths MUST be unique within the archive

As in web contexts, paths are keys, and do not entail the presence of intermediate directoriesm, or a file system. A resource with path `/music/jazz/coltrane.mp3` does not imply that `/music/` or `/music/jazz` exist as directories. However, clients unpacking archives and interpreting paths may choose to render resources to a file system with intermediate directories.

## Resource ordering

SZDT archive should be encoded in depth-first, first seen order to enable efficient streaming. Since archive memos always point to bytes, this means that an archive is made up of pairs of a memo block followed by a byte block.

The SZDT unarchiving CLI currently takes advantage of this structure to enable efficient streaming deserialization. Future versions may add logic to allow for streaming deserialization of arbitrary sequences.

## Appendix

### Reference Implementation

A reference implementation is available at [github.com/gordonbrander/szdt](https://github.com/gordonbrander/szdt).
