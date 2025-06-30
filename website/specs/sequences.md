---
layout: index.liquid
title: SZDT Sequences Specification
---

# SZDT Sequences Specification

SZDT Sequences can be used to store or deliver a series of content-addressed CBOR objects. Sequences serve as the base layer for higher-level SZDT formats like [archives](./archives).

## Sequence Format

SZDT Sequences are ordinary [CBOR sequences](https://www.rfc-editor.org/rfc/rfc8742.html)—streams of concatenated, definite-length CBOR values.

```
sequence = cbor_value | cbor_value | ...
```

Where each `cbor_value` is:
- A definite-length CBOR value
- Encoded using the [CBOR/c](#cbor-encoding) (CBOR Core) deterministic profile

## Content Addressing

Values in a sequence may reference other top-level values by content address, that is, the Blake3 hash of the CBOR/c-encoded value.

```
content_address = Blake3(cborc_encode(value))
```

- **Algorithm**: Blake3 with 256-bit (32-byte) output
- **Input**: The complete CBOR/c-encoded bytes of the object

### Verifying content addresses

When parsing sequences, implementors must verify each block by hashing the CBOR/c-encoded value with BLAKE3, and comparing the hash to the expected content address.

### Reference Format

Values may reference other top-level values by their content address. Content addresses are encoded as CBOR byte strings containing the 32-byte Blake3 hash:

```cbor
h'a1b2c3d4e5f6789012345678901234567890123456789012345678901234567890'
```

## Sequence ordering

Objects in sequences should be ordered using **depth-first, first-seen order** of content address traversal to facilitate efficient streaming.

## Incomplete DAGs

Implementors must not assume that all content-addressed objects will be included in the sequence. Authors may omit referenced objects from the sequence if desired.

## Duplication

Implementors must not assume that objects in sequences will be de-duplicated. The same object may appear multiple times in the same sequence.

## Multiple DAGs

A single sequence may contain multiple independent DAGs.

When multiple DAGs are present, authors should order objects so that the objects of the DAG are contiguous, and ordered in **depth-first, first-seen order** of content address traversal. That is, DAGs should be sequenced one after the other, and not interleaved.

## CBOR Encoding

All CBOR objects must use the deterministic [CBOR/c ("CBOR Core")](https://datatracker-ietf-org.lucaspardue.com/doc/draft-rundgren-cbor-core/) profile:

Deterministic encoding ensures:

- **Consistent hashing**: Identical logical objects produce identical hashes
- **Interoperability**: Different implementations produce identical byte streams
- **Verification**: Content can be independently verified

## Examples

### Simple Linear Chain

```cbor
// Object A (root, references B)
{
  "title": "Chain Example",
  "start": h'b_hash...'
}

// Object B (references C)
{
  "message": "Processing data",
  "next": h'c_hash...'
}

// Object C (leaf)
{
  "data": "Hello World",
  "value": 42
}
```

### Branching DAG

```cbor
// Object A (root, references B and C)
{
  "left": h'b_hash_32_bytes...',
  "right": h'c_hash_32_bytes...'
}

// Object B (references D)
{
  "ref": h'd_hash_32_bytes...'
}

// Object D (shared leaf)
{
  "value": "d"
}

// Object C (references D)
{
  "branch": "right",
  "ref": h'd_hash_32_bytes...'
}

// Object D (shared leaf duplicate)
{
  "value": "d"
}
```

### Multiple DAGs

```cbor
// DAG 1: Simple chain
// a1 -> a2
{
  "id": "a1",
  "next": h'b1_hash...'
}
{
  "id": "a2",
  "data": "end"
}

// DAG 2: Single object
{
  "id": "b1",
}
```

## Appendix

### MIME Type

SZDT sequences should use the MIME type:

```
application/vnd.szdt.seq+cbor-seq
```

### References

- [CBOR Sequences (RFC 8742)](https://www.rfc-editor.org/rfc/rfc8742.html)
- [CBOR Core Profile](https://datatracker-ietf-org.lucaspardue.com/doc/draft-rundgren-cbor-core/)
- [Blake3 Specification](https://github.com/BLAKE3-team/BLAKE3-specs)
