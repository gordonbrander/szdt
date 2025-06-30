---
layout: index.liquid
title: SZDT Archives Specification
---

# SZDT Archives Specification

SZDT Archives provide a standardized format for creating censorship-resistant signed file archives. SZDT archives can be distributed across multiple channels and verified without trusted authorities. Archives build on [SZDT Sequences]({{ "specs/sequences/" | prepend: site.url }}) and [SZDT Memos]({{ "specs/memos/" | prepend: site.url }}) to offer a container format for collections of files.

## Motivation

Traditional archives (ZIP, TAR, etc.) are vulnerable to tampering and provide no built-in authenticity verification.

SZDT Archives address these limitations by providing:

- **Cryptographic verification**: Every archive is signed and can be verified without trusted authorities
- **Decentralized distribution**: Archives can be shared via HTTP, BitTorrent, email, sneakernet, or any other transport
- **Censorship resistance**: No single point of failure or control

Archives are particularly valuable for:
- Scientific datasets that need long-term preservation
- Documentation that may face censorship
- Any collection of files that needs distributed, verifiable storage

## Archive Format

SZDT Archives are [CBOR sequences](https://www.rfc-editor.org/rfc/rfc8742.html) conforming to the [SZDT sequence]({{ "specs/sequences/" | prepend: site.url }}) spec, and having the following high-level structure:

```
archive = memo | manifest | bytes*
```

Where:
- **memo**: A signed [memo](./memos/) pointing to the manifest
- **manifest**: A CBOR-encoded manifest listing all resources in the archive
- **bytes**: CBOR byte strings, representing the raw bytes of each resource, in the order listed in the manifest

## Manifest Memo

The manifest memo is a standard [SZDT memo](./memos/) that wraps the archive manifest. It MUST contain:

### Required Protected Headers

| Field | Type | Description |
|-------|------|-------------|
| `iss` | String | Issuer DID of the archive creator |
| `iat` | Integer | Archive creation timestamp (Unix seconds) |
| `src` | Bytes(32) | Blake3 hash of the manifest CBOR |
| `content-type` | String | MUST be `"application/vnd.szdt.manifest+cbor"` |

### Optional Protected Headers

| Field | Type | Description |
|-------|------|-------------|
| `prev` | Bytes(32) | Blake3 hash of previous version's manifest memo |

### Example Manifest Memo

```cbor
{
  "protected": {
    "iss": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "iat": 1640995200,
    "src": h'b7e2c3d4f8a9b8c7d6e5f4g3h2i1j0k9l8m7n6o5p4q3r2s1t0u9v8w7x6y5z4a3b2',
    "content-type": "application/vnd.szdt.manifest+cbor",
  },
  "unprotected": {
    "sig": h'a1b2c3d4e5f6789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890'
  }
}
```

## Manifest Structure

The manifest is a CBOR map that describes all resources in the archive, in the order they will subsequently appear in the CBOR Sequence:

```cbor
{
  "resources": [
    {
      "src": h'abcd1234...',
      "length": 1024,
      "path": "/folder/file.txt",
      "content-type": "text/plain"
    },
    // ... more resources
  ]
}
```

### Manifest Schema

| Field | Type | Description |
|-------|------|-------------|
| `resources` | Array | List of resource objects in archive order |

### Resource Schema

Each resource object describes a single file in the archive:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `src` | Bytes(32) | Yes | Blake3 hash of the resource content |
| `length` | Integer | Yes | Length of resource in bytes |
| `path` | String | Yes | path within archive |
| `content-type` | String | No | MIME type of the resource |

### Path Requirements

- Paths MUST start with "/" (absolute within archive)
- Paths MUST use "/" as separator (Unix-style)
- Paths MUST NOT contain ".." components
- Paths MUST be unique within the archive

As in web contexts, paths are keys, and do not entail the presence of intermediate directoriesm, or a file system. A resource with path `/music/jazz/coltrane.mp3` does not imply that `/music/` or `/music/jazz` exist as directories. However, clients unpacking archives and interpreting paths may choose to render resources to a file system with intermediate directories.

### Resource Ordering

Resources in the manifest MUST appear in the same order as their corresponding sequences in the archive. This enables:

- **Random access**: Calculate offsets via position and length
- **Streaming**: Process resources as they arrive

## Random Access

Archives support efficient random access to individual files:

1. **Read manifest memo**: Parse first CBOR item and verify signature
2. **Read manifest**: Parse second CBOR item and verify against memo's `src` hash
3. **Find resource**: Locate desired file by path or hash in manifest
4. **Calculate offset**: Sum byte lengths of memo, manifest, and all preceding resources
5. **Seek and read**: Navigate to offset and read `length` bytes
6. **Verify content**: Compare Blake3 hash with resource's `src` field

```
resource_offset = memo_size + manifest_size + sum(preceding_resource_lengths)
```

Where:
- `memo_size`: Byte size of the manifest memo CBOR
- `manifest_size`: Byte size of the manifest CBOR
- `preceding_resource_lengths`: Sum of `length` fields for all resources before the target

## Versioning

Archives may be versioned using the `prev` field and versioning semantics described in the [SZDT memos specification]({{ "specs/memos/" | prepend: site.url }}).

## Security Considerations

### Archive Integrity

- Verify manifest memo signature before processing
- Verify manifest hash against memo's `src` field
- Verify each resource hash during extraction
- Reject archives with invalid timestamps or signatures

### Path Traversal Prevention

- Validate all paths start with "/"
- Reject paths containing ".." components
- Sanitize paths before filesystem operations
- Consider path length limits

## Examples

### Simple Text Archive

```cbor
// Manifest memo
{
  "unprotected": {
    "sig": h'5d2f8a0f3b4c7e8f9a1b2c3d4e5f6789abcdef01234567890abcdef0123456789abcdef01234567890abcdef0123456789abcdef01234567890abcdef'
  },
  "protected": {
    "iss": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "iat": 1640995200,
    "src": h'a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3',
    "content-type": "application/vnd.szdt.manifest+cbor",
    "title": "Hello World Archive"
  }
}

// Manifest
{
  "resources": [
    {
      "src": h'c8d5e6f7a9b8c7d6e5f4g3h2i1j0k9l8m7n6o5p4q3r2s1t0u9v8w7x6y5z4a3b2',
      "length": 11,
      "path": "/hello.txt",
      "content-type": "text/plain"
    }
  ]
}

// Resource bytes
h'48656c6c6f20576f726c64'  // "Hello World"
```

## Appendix

### MIME Type

SZDT archives SHOULD use the MIME type:

```
application/vnd.szdt.archive+cbor-seq
```

### File Extension

SZDT archives SHOULD use the file extension:

```
.szdt
```

### Reference Implementation

A reference implementation is available at [github.com/gordonbrander/szdt](https://github.com/gordonbrander/szdt).
