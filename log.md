# Log

Reverse-chronological.

---

Always sign over the hash of the (unmodified) bytes. This way the claim can be distributed separately from the bytes, and the bytes can be retreived over content-addressed storage.

---

Alternatively, we could separate the memo from the bytes, and hold a hash of the body part in the memo under `src` or `body`. If we did this, we should flatten the graph into a CBOR sequence (like CAR).

In what order? Depth-first, first-seen order.

> A filecoin-deterministic car-file is currently implementation-defined as containing all DAG-forming blocks in first-seen order, as a result of a depth-first DAG traversal starting from a single root. <https://ipld.io/specs/transport/car/carv1/#determinism>

Since we aren't doing deep graphs, this would mean:

```
headers | body | headers | body
```

---

Split headers into protected/unprotected (like COSE):

```
[
  {
    sig: h`abc123`
  }, // unprotected
  {
    "content-type": "application/octet-stream",
  }, // protected
  h'xyz123' // body
]
```

Signatures happen over protected headers and body:

```
sign(
  [
    {
      "content-type": "application/octet-stream",
    },
    h'xyz123'
  ],
  key
)
```

If we split headers into protected and unprotected, then we can permissionlessly extend signing and encryption features in future, without resorting to spec'ing a signing procedure beyond the protected/unprotected mechanism. For example, we could add witnesses by signing over the protected fields, and adding a `witnesses` field to the unprotected headers.

---

What should we index?

In IPLD CARv2 format, the index provides an index of CIDs (Content Identifiers) to their byte offsets within the CAR file. Maybe that's all we need.

For the case where you want to e.g. to serve a single memo (or a slice) from an archive, without unpacking the archive, what you need is a map of hashes to byte offsets.

The thing we want to sign over for the collection is probably a hash sequence (concatenated 32-byte Blake3 hashes).

---

New idea for archive format, leveraging Blake3 and Bao for streaming verification.

- Archives are CBOR sequences of memos
  - Everything is a memo
- Each memo is an array of `[headers, bytes]`, where headers are a CBOR map of metadata, and bytes are the body part.
  - Memos can be signed
  - Signature lives on `sig` field in the headers
    - Signature verification happens by reconstructing the unsigned memo
      - Copy memo, remove `sig` header, and verify
      - Or perhaps we should sign over headers only and include a hash of the body in the headers.
    - Signature in header allows for Blake3 streaming verification
  - Memos are encoded using cbor/c (CBOR core) profile. All CBOR items must be fixed length.
- A manifest may be generated over archives
  - Manifest can be distributed alone, sidecar, or at the head of the CBOR sequence
  - The manifest corresponds to a particular CBOR sequence
  - Each manifest entry contains `{path, length, hash}`
    - `path` is a string representing a file path
    - `length` is an integer representing the length of the referenced memo in bytes
    - `hash` is a string representing the Blake3 hash of the referenced memo
  - Entries are in order of their appearance in the sequence
    - If you have the manifest and the sequence, you can efficiently access any memo by its path by seeking to the appropriate offset, calculated as the sum of the lengths of all previous memos, and then reading `length` bytes.

What this enables:

- Efficient streaming
- Efficient random access (via optional manifest)
- Streaming verification of memo integrity
