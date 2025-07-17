# Log

Reverse-chronological.

---

Noosphere stored the private key in:

```
~/.noosphere/keys/<keyname>.public
~/.noosphere/keys/<keyname>.private
```

With
- private key being stored as a mnemonic
- public key stored as `did:key`

---

How are keys sorted in...

- CBOR Core: bytewise lexicographic order of their deterministic encoding <https://www.ietf.org/archive/id/draft-rundgren-cbor-core-11.html#section-2.2>
- cborc42: bytewise lexicographic order <https://datatracker.ietf.org/doc/html/draft-caballero-cbor-cborc42-00#section-2.2>
- dag-cbor bytewise lexicographic order (I think?) <https://ipld.io/specs/codecs/dag-cbor/spec/>
  - serde_ipld_dagcbor <https://github.com/ipld/serde_ipld_dagcbor/blob/d7f93d06ce6a19867abeea78d02d6ded6c476b81/src/ser.rs#L466>

---

https://dasl.ing/masl.html has two modes:

- `src` points to a single resource
- `resources` is a map of n resources by path
  - Each resource value is a map representing its own headers

We could do something like this, where each entry has a header of `length` and `start` or `position`. The only difficulty is that we need to include the length of the manifest somehow if this is going to work for HTTP range requests. I think that means that the index HAS to come at the end, which presents its own challenges.

---

Serialization options:

- Detatched headers with flattened DAG
  - `{protected, unprotected} body`
  - Can immediately verify signature, then streaming verify body
  - Less convenient when reading back in
- Memo, sign over headers, with body hash in headers
  - `[{protected, unprotected}, body]`
  - `[protected, unprotected, body]`
  - Has benefits of DAG approach
  - But not all CBOR parsers support streaming
- Memo, sign over headers and (hashed) body
  - `{protected, unprotected} body`
  - Disadvantage: Can't streaming verify body because there is no hash to
    compare to.
  - Disadvantage: Can't verify signature until body is hashed
- Memo, sign over headers and hashed body w footer
  - `[headers, body, footer]`
  - Hash headers
  - Hash body
  - Verify signature against hashseq of `[headers, body]`
  - Disadvantage: Can't streaming verify body because there is no hash to
    compare to.
  - Disadvantage: Can't verify signature until body is hashed

Tradeoffs:

- With footer approach, I can streaming hash, then verify
  - But I can't streaming verify, since there is no comparison hash
  - I have to wait until end to verify signature
- With DAG approach, I can verify sig, then streaming verify hash

Decisions:

- I think DAG approach is overall better, if a little more complex.
- Regardless, we should sign over the headers, and the header should contain the body hash.
  - This allows us to do streaming verification by comparing the body hash to the hash in the headers

Signing Phase:

- Compute the complete Blake3 hash tree over the entire data
- Sign the root hash with your private key
- Store/transmit: signature + Blake3 tree nodes needed for verification

Streaming Verification Phase:

- Verifier receives and validates the signature on the root hash immediately
- As data chunks arrive, they come with their Blake3 proof path (sibling hashes up to the root)
- Each chunk can be verified against the already-trusted root hash
- Any tampering is detected immediately for that chunk

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
