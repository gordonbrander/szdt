# Log

Reverse-chronological.

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
