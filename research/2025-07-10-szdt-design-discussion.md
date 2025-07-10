# SZDT Design Discussion - July 10, 2025

## Initial Question: Memo Structure Design

**User**: I want to think with you about the design of SZDT memos and sequences. SZDT is inspired by IPFS/IPLD, Bluesky, and Nostr. It aims to be a simple protocol for self-certifying, signed, content-addressed data. The primary usecase is as a way to make app data decentralizable. It's meant to be fairly open-ended. 

The question I want to work through is about the overall structure. Currently, memos point to the body part via the src field. They don't actually contain the body. However, an alternative design could be to rename Memo -> Headers, and then Memo becomes a CBOR structure of [Headers, Body]. 

Some pros and cons I can see: currently, if you don't know what the shape of a sequence is, you need to first deserialize a block to a CBOR Value in Rust, then use context queues to figure out what that value actually is. If you're working with a fixed set of types, this might be improved with Serde's enum-based deserialization. An alternative design might make Memos of [Headers, Body] always be the top-level structure. Body is always CBOR bytes, and the content-type header tells you what to do with it. However, I'm not sure that actually saves anything efficiency-wise, since we have to store the bytes in memory, and then deserialize it, storing it in memory again. Is this actually an improvement on deserializing to a CBOR Value type, or using the enum approach?

**Assistant**: Looking at your current SZDT design and the alternative you're considering, I think the current approach with separate memo headers and body content is actually more efficient and flexible than the [Headers, Body] alternative. Here's why:

## Current Design Advantages

**Memory Efficiency**: Your current design is more memory-efficient because:
- You can stream and verify the body incrementally with Blake3/Bao
- The body doesn't need to be held in memory alongside the memo
- Multiple memos can reference the same body content without duplication

**Deserialization Efficiency**: For the "unknown sequence shape" problem, you have better options with the current design:
- **Enum approach**: You can absolutely use Serde's enum-based deserialization with your current structure. Define an enum of known memo types and deserialize directly to the appropriate variant.
- **Streaming**: You can deserialize and process memos as they arrive, then fetch bodies on-demand.

## Alternative Design Drawbacks

The [Headers, Body] approach you're considering would actually be **less efficient**:

```rust
// Alternative: [Headers, Body] - requires two deserializations
let memo: [Headers, Vec<u8>] = deserialize(bytes)?;  // First deserialization
let body: ActualBodyType = deserialize(memo.1)?;     // Second deserialization
```

This doesn't save memory or CPU compared to your current approach - it just moves the indirection from content addressing to embedded bytes.

## Recommended Approach

For handling unknown sequence shapes with your current design, consider:

```rust
// Define your known memo types
#[derive(Deserialize)]
#[serde(tag = "content-type")]
enum KnownMemoTypes {
    #[serde(rename = "text/plain")]
    TextPlain(TextMemo),
    #[serde(rename = "application/json")]
    Json(JsonMemo),
    // ... other types
}

// Fallback to generic for unknown types
let memo = match deserialize::<KnownMemoTypes>(bytes) {
    Ok(known) => known,
    Err(_) => deserialize::<GenericMemo>(bytes)?,
};
```

Your current design's separation of concerns (memo metadata vs. body content) aligns well with SZDT's goals of content addressing, streaming verification, and flexible distribution. The alternative would sacrifice these benefits without meaningful efficiency gains.

## Clarification on Streaming Verification

**User**: A note of clarification: I don't think we actually lose the ability to do streaming verification with the [Headers, Body] design. This is because the Headers still contain a `src` field, which gets signed over. The body can then be streaming verified against the hash. The difference is in where the bytes live. So in the current design, a CBOR sequence might look like: `Memo bytes Memo bytes...`, whereas in the alternative design, a CBOR sequence might look like `[Headers, bytes] [Headers, bytes]...`. The difficulty, as I understand it, in the first design, is that you're not guaranteed that the body bytes come after the memo. A sequence might omit blocks, for example. This might lead to some difficulties in deserializing, because how do you know what to deserialize a block to? The block could be any CBOR value. I might be missing something about what Serde can do here BTW... By contrast, in the second design, the body bytes always follow the headers because they're part of the same structure. The content-type will tell you how to deserialize the body.

**Assistant**: You're absolutely right about the deserialization ambiguity problem! I missed that key issue.

## The Deserialization Challenge

In the current design, when you encounter a CBOR block in a sequence, you face the "what is this?" problem:

```rust
// Current design: what is this block?
let block: serde_cbor::Value = deserialize(bytes)?;
// Is it a Memo? Body content? Which type of body content?
```

You'd need some kind of type envelope or convention to distinguish memo blocks from body blocks. Even with Serde enums, you'd still need to know "is this a memo or body content?" before choosing how to deserialize.

## Alternative Design Clarity

The `[Headers, Body]` approach eliminates this ambiguity:

```rust
// Alternative: always know the structure
let memo: [Headers, Vec<u8>] = deserialize(bytes)?;
let headers = memo.0;
let body_bytes = memo.1;

// content-type tells you how to interpret body_bytes
match headers.content_type {
    "text/plain" => String::from_utf8(body_bytes)?,
    "application/json" => serde_json::from_slice(&body_bytes)?,
    // ...
}
```

## The Real Tradeoff

You're right that both designs can do streaming verification. The real tradeoff is:

**Current design**: 
- More flexible distribution (memo and body can be separated)
- Parsing ambiguity in sequences

**Alternative design**:
- Self-contained parsing (no ambiguity)
- Body always bundled with headers

Given SZDT's goals, the alternative might actually be better for the core use case of "decentralizable app data" where you typically want the complete record together anyway. You could still achieve separation by having sequences that contain just headers (with body distributed separately), but the default case would be unambiguous.

The parsing clarity benefit seems significant enough to outweigh the bundling constraint.

## Question About Serde Capabilities

**User**: Does Serde have a way to handle the first design? Imagine we anticipate two block types: Memo and CBOR bytes. Can Serde be used to automatically suss out which one we're seeing and deserialize it?

**Assistant**: Yes, Serde can handle this! There are several approaches, each with different tradeoffs:

## Option 1: Untagged Enum (Simplest)

```rust
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum CborItem {
    Memo(Memo),
    RawValue(serde_cbor::Value),
}
```

This works because:
- Serde tries `Memo` first (requires `protected` and `unprotected` fields)
- If that fails, falls back to `RawValue` (accepts any CBOR)
- Your `Memo` has a distinctive structure that won't match arbitrary CBOR

## Option 2: Try-Parse Pattern (Better Performance)

```rust
fn parse_cbor_item(data: &[u8]) -> Result<CborItem, Error> {
    // Try memo first
    if let Ok(memo) = serde_cbor::from_slice::<Memo>(data) {
        return Ok(CborItem::Memo(memo));
    }
    
    // Fall back to generic value
    let value = serde_cbor::from_slice::<serde_cbor::Value>(data)?;
    Ok(CborItem::RawValue(value))
}
```

## Option 3: Structural Detection (Most Robust)

```rust
impl CborItem {
    pub fn from_cbor_bytes(data: &[u8]) -> Result<Self, Error> {
        let value: serde_cbor::Value = serde_cbor::from_slice(data)?;
        
        // Check if it looks like a Memo
        if Self::looks_like_memo(&value) {
            if let Ok(memo) = serde_cbor::from_slice::<Memo>(data) {
                return Ok(CborItem::Memo(memo));
            }
        }
        
        Ok(CborItem::RawValue(value))
    }
    
    fn looks_like_memo(value: &serde_cbor::Value) -> bool {
        match value {
            serde_cbor::Value::Map(map) => {
                map.contains_key(&serde_cbor::Value::Text("protected".to_string())) &&
                map.contains_key(&serde_cbor::Value::Text("unprotected".to_string()))
            }
            _ => false,
        }
    }
}
```

## Performance Comparison

1. **Option 1 (Untagged)**: Simplest but can be 2-5x slower
2. **Option 2 (Try-Parse)**: Good balance of simplicity and performance  
3. **Option 3 (Structural)**: Most efficient, avoids failed deserializations

For your use case, I'd recommend **Option 2** or **Option 3** since your `Memo` structure is distinctive enough to detect reliably, and performance matters for parsing sequences.

So yes, Serde can absolutely handle this! The current design's parsing ambiguity is solvable without changing to the `[Headers, Body]` approach.

## Double Allocation Concern

**User**: The structural approach looks compelling, but I think that means I must allocate bytes twice (once for the Value, once for the deserialized struct). I guess that's not so different on balance from the [Headers, Body] approach, which also has to allocate the bytes for the body, and then again when deserializing the body. OTOH, perhaps this is splitting hairs if it's all happening on the stack?

**Assistant**: You're absolutely right to think about the allocation patterns! Let me break down what's happening:

## Structural Approach Allocations
```rust
// First allocation: Value tree (heap allocated)
let value: serde_cbor::Value = serde_cbor::from_slice(data)?;

// Second allocation: Memo struct (heap allocated)  
let memo: Memo = serde_cbor::from_slice::<Memo>(data)?;
```

You're parsing the **same CBOR bytes twice**, allocating:
1. `Value` tree (with all its `String`s, `Vec`s, etc.)
2. `Memo` struct (with its own `String`s, `Vec`s, etc.)

## [Headers, Body] Approach Allocations
```rust
// First allocation: Headers + Vec<u8> for body
let [headers, body_bytes]: [Headers, Vec<u8>] = serde_cbor::from_slice(data)?;

// Second allocation: when deserializing body
let body: ActualBodyType = serde_json::from_slice(&body_bytes)?;
```

You're parsing **different data twice**, allocating:
1. `Headers` + `Vec<u8>` copy of body bytes  
2. Deserialized body struct

## The Key Difference

**Structural approach**: Redundant CBOR parsing (CPU waste)
**[Headers, Body]**: No redundant parsing, just type-specific parsing

You're right that both do two allocations, but the `[Headers, Body]` approach is probably more efficient because:

1. **Less CPU overhead**: No redundant CBOR parsing
2. **Body bytes are pre-extracted**: Ready for type-specific parsing
3. **Cleaner separation**: Headers parsed once, body parsed when needed

## Zero-Copy Alternative?

If you want to avoid the double allocation in the structural approach, you could use a streaming parser that doesn't build the intermediate `Value`:

```rust
// Hypothetical streaming approach - check if bytes look like memo without full parse
fn peek_memo_structure(data: &[u8]) -> bool {
    // Use a streaming CBOR parser to check for "protected"/"unprotected" keys
    // without building the full Value tree
}
```

But this would be more complex to implement and might not be worth it.

Given this analysis, the `[Headers, Body]` approach starts looking more attractive - it eliminates the redundant parsing overhead while keeping the allocation count the same.

## The Persistent Ambiguity Problem

**User**: OTOH even with the [Headers, Body] approach, it should be valid to place arbitrary CBOR values within a CBOR sequence, since these might still be content addressed from within the memo or its body.

**Assistant**: Excellent point! You're absolutely right. Even with the `[Headers, Body]` approach, you'd still need to handle arbitrary CBOR values in sequences because:

1. **Content-addressed references**: Memos can reference external CBOR objects via `src` or other hash fields
2. **Partial sequences**: Not every block needs to be a complete `[Headers, Body]` memo
3. **Mixed content**: You might have standalone referenced objects alongside complete memos

So you're back to the same parsing ambiguity problem:

```rust
// CBOR sequence with [Headers, Body] approach
[Headers, Body] | arbitrary_cbor_object | [Headers, Body] | another_object...
//                 ^                                        ^
//                 How do you know what these are?
```

## The Parsing Problem Persists

```rust
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum CborItem {
    CompleteMemo([Headers, Vec<u8>]),  // [Headers, Body] structure
    ArbitraryValue(serde_cbor::Value), // Content-addressed objects
}
```

You'd still need the same structural detection or try-parse approaches to distinguish between:
- Complete `[Headers, Body]` memos
- Standalone content-addressed objects

## The Real Insight

This actually **strengthens the case for keeping the current design**! Because:

1. **Parsing ambiguity exists either way**
2. **Current design is more flexible**: Memo and body can be distributed separately
3. **Performance is similar**: Both approaches need the same detection logic
4. **Streaming benefits**: Current design preserves the ability to stream-verify large bodies independently

The `[Headers, Body]` approach doesn't actually solve the fundamental "what type is this CBOR block?" problem—it just changes which types you need to distinguish between.

Given that you'll need sophisticated parsing logic regardless, the current design's flexibility advantages become more compelling. The parsing complexity is unavoidable, so you might as well keep the more flexible content model.

## The Unsolved Dilemma

**User**: so I'm not sure we've actually solved the dilemma

**Assistant**: You're absolutely right - we haven't solved the fundamental dilemma! 

Both designs face the **same core problem**: when you encounter a CBOR block in a sequence, you don't know what it is. The parsing ambiguity exists regardless of the structure.

## The Unchanged Problem

**Current design**: `Memo | body_bytes | Memo | arbitrary_cbor | ...`
- Need to distinguish: Memo vs. body content vs. other CBOR

**[Headers, Body] design**: `[Headers, Body] | arbitrary_cbor | [Headers, Body] | ...`  
- Need to distinguish: Complete memo vs. arbitrary CBOR

## The Real Question

Since parsing complexity is unavoidable, the decision comes down to:

1. **Flexibility vs. Simplicity**: Current design allows memo/body separation, but [Headers, Body] guarantees they're together
2. **Distribution patterns**: Do you typically want memos and bodies bundled or separated?
3. **Implementation complexity**: Are there other factors that tip the scales?

## Possible Solutions to the Dilemma

1. **Type tags**: Add a CBOR tag to distinguish block types
2. **Sequence metadata**: Start sequences with a header describing block types
3. **Accept the ambiguity**: Use structural detection and live with the complexity
4. **Hybrid approach**: Support both formats depending on use case

The structural parsing approaches we discussed work for either design, but they don't eliminate the fundamental "what is this block?" problem. You're right that we're still stuck with the original dilemma about which design to choose.

What's your intuition about how SZDT data will typically be distributed? Always bundled, always separated, or mixed?

## General-Purpose Format Considerations

**User**: It's a general-purpose format, like HTTP, so there's no telling. Some use cases might stream only metadata (headers), others might stream headers and body content. It's also possible that the content might be CBOR structures that reference other CBOR structures via content addressing (similar to IPLD). Further, there may be cases where people use SZDT with content addressed storage.

**Assistant**: You've hit the nail on the head - this is exactly the kind of design challenge that general-purpose formats face. Given that flexibility, I think the current design is actually the right choice.

## Why Current Design Wins for General-Purpose Use

**Maximum flexibility**: 
- Headers-only streams for metadata
- Bundled sequences for complete records  
- Mixed sequences with content-addressed references
- Compatibility with content-addressed storage systems

**IPLD-like capabilities**: Content addressing naturally supports complex DAG structures where CBOR objects reference other CBOR objects.

**Transport independence**: The memo/body separation means you can distribute them via different channels (memo via fast metadata API, body via CDN).

## The Parsing Ambiguity is Just Life

For general-purpose formats, parsing ambiguity is inevitable. Look at:
- **HTTP**: Need to parse headers to know body format
- **IPLD**: Need to examine structure to determine data types
- **Email**: MIME boundaries and content-type headers

The structural detection approach we discussed is actually standard practice. You're not avoiding complexity by changing the design - you're just moving it around.

## Accept the Complexity, Embrace the Flexibility

The current design aligns perfectly with SZDT's goals:
- Self-certifying (signatures over content addresses)
- Transport-agnostic (separate distribution of memo/body)
- Content-addressed (supports complex DAG structures)
- Streaming-friendly (incremental verification)

The `[Headers, Body]` approach would sacrifice these benefits to solve a parsing problem that... still exists anyway.

Your instinct to keep the current design seems right. The parsing complexity is manageable with the techniques we discussed, and the flexibility is worth it for a general-purpose format.

## Adding a Type Discriminator

**User**: Ok, given this is the case, would it be a good idea to add a type field to memos? I imagine something like a type property at the top level of the memo that always has the value "szdt/memo", or something along those lines.

**Assistant**: That's a brilliant idea! Adding a type discriminator would solve the parsing ambiguity problem elegantly.

## Proposed Structure

```cbor
{
  "type": "szdt/memo",
  "protected": {
    "iss": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "iat": 1640995200,
    "src": h'c8d5e6f7...',
    "content-type": "text/plain"
  },
  "unprotected": {
    "sig": h'5d2f8a0f...'
  }
}
```

## Benefits

**Unambiguous parsing**: Instantly know if a CBOR block is a memo
**Future-proof**: Could add other SZDT types like `"szdt/archive"` or `"szdt/index"`
**Standards alignment**: Similar to JSON-LD's `@type` or ActivityPub's `type`
**Tooling-friendly**: Generic CBOR viewers can display type information

## Implementation

```rust
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum CborItem {
    SzdtMemo {
        #[serde(rename = "type")]
        type_field: String, // Must be "szdt/memo"
        protected: ProtectedHeaders,
        unprotected: UnprotectedHeaders,
    },
    ArbitraryValue(serde_cbor::Value),
}
```

Or even simpler:

```rust
fn parse_cbor_item(data: &[u8]) -> Result<CborItem, Error> {
    let value: serde_cbor::Value = serde_cbor::from_slice(data)?;
    
    if let Some(type_str) = get_type_field(&value) {
        match type_str.as_str() {
            "szdt/memo" => Ok(CborItem::Memo(serde_cbor::from_slice(data)?)),
            _ => Ok(CborItem::ArbitraryValue(value)),
        }
    } else {
        Ok(CborItem::ArbitraryValue(value))
    }
}
```

## Design Questions

1. **Top-level or protected?** I'd suggest top-level for parsing efficiency
2. **Naming**: `"szdt/memo"` follows MIME-type conventions
3. **Required vs optional**: Should probably be required for new memos, optional for backward compatibility

This would make SZDT sequences much more deterministic to parse while preserving all the flexibility benefits of the current design.

## MIME Types vs Simple Strings

**User**: We use mime types for the content-type field of memos. Should the type value also be a mime type (e.g. application/vnd.szdt.memo+cbor)? What's the typical approach here? It would be nice to use just one style of type flag, but are mime types typically used in-band like this, or are they meant to be used out-of-band?

**Assistant**: Great question! MIME types are typically used **out-of-band** (HTTP headers, file extensions, etc.), not as in-band discriminators. Here's the typical approach:

## MIME Types: Out-of-Band

```http
Content-Type: application/vnd.szdt.memo+cbor
```

MIME types describe the **entire payload format** to the transport layer, not individual objects within the payload.

## In-Band Discriminators: Simple Strings

Most formats use simple strings for in-band type discrimination:

**JSON-LD**: `"@type": "Person"`
**ActivityPub**: `"type": "Note"`
**IPLD**: `"@type": "dag-pb"`
**Docker**: `"mediaType": "application/vnd.docker.image.manifest.v2+json"`

## Recommended Approach

Use both, but for different purposes:

```cbor
{
  "type": "memo",           // In-band discriminator (simple string)
  "protected": {
    "content-type": "text/plain",  // MIME type for body content
    ...
  },
  ...
}
```

**Why this works better:**
- `"type": "memo"` - tells parsers "this is a memo object"
- `"content-type": "text/plain"` - tells apps "the body is plain text"
- Transport layer uses `application/vnd.szdt.seq+cbor-seq` for the entire sequence

## Namespace Options

If you want to avoid collisions:
- `"type": "szdt:memo"` (URI-like)
- `"type": "szdt/memo"` (path-like)
- `"type": "memo"` (simple, assumes SZDT context)

I'd lean toward `"type": "memo"` since you're already in an SZDT context. Simple and clean.

## Versioning Considerations

**User**: or maybe I should use doctype as the key? That seems to be used in HTML. It would be nice if the discriminator could provide an escape hatch for versioning the way doctypes are versioned in HTML. What's the typical approach here?

**Assistant**: Great idea! `doctype` is indeed used for format versioning and would provide the escape hatch you're looking for.

## HTML Doctype Pattern

HTML uses doctype for both format identification and versioning:

```html
<!DOCTYPE html>                           <!-- HTML5 -->
<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01//EN" "http://www.w3.org/TR/html4/strict.dtd">
```

## Common Versioning Approaches

**JSON Schema**: `"$schema": "https://json-schema.org/draft/2020-12/schema"`
**OpenAPI**: `"openapi": "3.0.0"`
**Docker**: `"schemaVersion": 2`
**ActivityPub**: `"@context": "https://www.w3.org/ns/activitystreams"`

## Recommended Approach for SZDT

```cbor
{
  "doctype": "szdt/memo/1.0",
  "protected": {
    "iss": "did:key:z6Mk...",
    "iat": 1640995200,
    ...
  },
  ...
}
```

**Benefits:**
- **Version escape hatch**: `szdt/memo/1.0` → `szdt/memo/2.0`
- **Format identification**: Clear this is an SZDT memo
- **Familiar pattern**: Similar to HTML doctype
- **Extensible**: Could add `szdt/archive/1.0`, etc.

## Alternative Formats

```cbor
"doctype": "szdt:memo:1.0"     // URI-like
"doctype": "szdt.memo.v1"      // Dotted
"doctype": "szdt/memo@1.0"     // NPM-style
```

## Implementation Benefits

```rust
fn parse_cbor_item(data: &[u8]) -> Result<CborItem, Error> {
    let value: serde_cbor::Value = serde_cbor::from_slice(data)?;
    
    if let Some(doctype) = get_doctype(&value) {
        match doctype.as_str() {
            "szdt/memo/1.0" => Ok(CborItem::MemoV1(serde_cbor::from_slice(data)?)),
            "szdt/memo/2.0" => Ok(CborItem::MemoV2(serde_cbor::from_slice(data)?)),
            _ => Ok(CborItem::Unknown(value)),
        }
    } else {
        Ok(CborItem::ArbitraryValue(value))
    }
}
```

This gives you both format identification AND versioning in one field, following established web standards patterns.

## Survey of Other Protocols

**User**: Are there other structured data protocols based on JSON/CBOR/YAML/etc that do this? Or is it unusual?

**Assistant**: You're asking exactly the right question! Let me survey what other structured data protocols actually do:

## Protocols That Use In-Band Type Discriminators

**JSON-LD**: `"@type": "Person"`
**ActivityPub**: `"type": "Note"` 
**JSON Schema**: `"$schema": "https://json-schema.org/draft/2020-12/schema"`
**OpenAPI**: `"openapi": "3.0.0"`
**Docker Manifests**: `"mediaType": "application/vnd.docker.image.manifest.v2+json"`
**Kubernetes**: `"apiVersion": "v1", "kind": "Pod"`
**CloudFormation**: `"AWSTemplateFormatVersion": "2010-09-09"`

## Protocols That Don't

**HTTP**: Relies on `Content-Type` header (out-of-band)
**IPLD**: Uses codec information in CID, not payload
**Protocol Buffers**: Uses schema definitions, not in-band typing
**Avro**: Schema-based, not self-describing
**Most CBOR uses**: Relies on context or external typing

## The Pattern

**Self-describing formats** (JSON-LD, ActivityPub, OpenAPI) tend to use in-band discriminators because they're designed to be parsed without external context.

**Schema-based formats** (Protobuf, Avro) don't need them because the schema provides the type information.

**Transport protocols** (HTTP, IPLD) use out-of-band typing because they're moving typed payloads.

## SZDT's Position

SZDT is more like JSON-LD/ActivityPub - it's designed to be self-describing and parseable without external schema. The `doctype` field would be **totally normal** for this category.

## Real-World Example

ActivityPub objects always include type information:

```json
{
  "type": "Note",
  "id": "https://example.com/note/1",
  "content": "Hello world"
}
```

This is considered good practice for self-describing formats.

**Verdict**: Adding `doctype` to SZDT memos would be following established patterns, not unusual at all. It's the standard approach for self-describing structured data formats.

## Field Name Choice

**User**: What's the most reasonable/unsurprising fieldname for a type discriminator? Type? Kind? Doctype?

**Assistant**: Looking at the survey of real-world usage:

## Most Common: `"type"`

**Very common:**
- ActivityPub: `"type": "Note"`
- JSON-LD: `"@type": "Person"` 
- Docker: `"mediaType": "..."`
- GeoJSON: `"type": "Feature"`

**Common:**
- Kubernetes: `"kind": "Pod"`
- OpenAPI: `"openapi": "3.0.0"`
- JSON Schema: `"$schema": "..."`

## The Winner: `"type"`

`"type"` is by far the most common and unsurprising choice. It's used in:
- ActivityPub (federated social web)
- GeoJSON (geographic data)
- JSON-LD (linked data)
- Many ad-hoc JSON APIs

## Why Not the Others?

**`"kind"`**: Kubernetes-specific terminology, less universal
**`"doctype"`**: HTML-specific, might confuse web developers
**`"schema"`**: Usually refers to validation schemas, not type discrimination

## Recommendation

```cbor
{
  "type": "szdt/memo/1.0",
  "protected": { ... },
  "unprotected": { ... }
}
```

**Benefits:**
- Most familiar to developers
- Shortest/cleanest
- Follows established JSON/web conventions
- Works well with Serde's `#[serde(tag = "type")]`

If you want to be extra explicit about versioning, you could use `"type"` for the base type and add a separate version field, but the combined approach is more common and cleaner.

## Content Address Complexity

**User**: Another question along these lines. IPFS/IPLD add a lot of prefix bytes to CIDs to signal base encoding, media type, hash type, etc. I want to get away from this complexity. Instead, I've been using plain Blake3 hashes as content addresses. However, I'm wondering if it would be worth having in-band information signaling these bytes are a hash.

**Assistant**: I see you've already added `kind` to the Memo struct! Great choice - it's a good middle ground between `type` and `doctype`.

On your IPFS/IPLD question about content addresses:

## The IPFS/IPLD Complexity Problem

IPFS CIDs are indeed complex:
```
bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi
```
That encodes: base32, CID version, codec, hash function, hash length, and the actual hash.

## Your Blake3 Approach is Cleaner

Plain Blake3 hashes are much simpler:
```
c8d5e6f7a9b8c7d6e5f4g3h2i1j0k9l8m7n6o5p4q3r2s1t0u9v8w7x6y5z4a3b2c1
```

## Should You Add In-Band Hash Signaling?

I'd say **no** for these reasons:

**1. Keep it simple**: Your goal is to avoid IPFS complexity
**2. Context is sufficient**: In SZDT, hash fields are clearly labeled (`src`, `prev`, etc.)
**3. Single hash function**: Blake3 is your only hash function, so no ambiguity
**4. CBOR typing**: Hashes are already typed as `Bytes(32)` in CBOR

## IPFS vs SZDT Context

**IPFS**: Needs complex CIDs because:
- Multiple hash functions (SHA256, Blake2b, Blake3...)
- Multiple content types in the same address space
- Needs to be self-describing across all protocols

**SZDT**: Doesn't need this because:
- Single hash function (Blake3)
- Context from field names (`src` = content hash, `prev` = memo hash)
- Controlled format (you know you're in SZDT)

## Recommendation

Keep your plain Blake3 approach. The field names provide sufficient context:

```cbor
{
  "kind": "szdt/memo/1.0",
  "protected": {
    "src": h'c8d5e6f7...',    // Obviously a content hash
    "prev": h'a1b2c3d4...',   // Obviously a memo hash
    ...
  }
}
```

This is much cleaner than IPFS while still being unambiguous. The complexity reduction is a feature, not a bug.

## Conclusion

This discussion explored the fundamental design choices for SZDT memos, weighing flexibility against parsing complexity. The key insights were:

1. **Parsing ambiguity is unavoidable** for general-purpose formats that support mixed content types
2. **Current design provides maximum flexibility** for different distribution patterns
3. **Adding a type discriminator** (settled on `"kind"`) elegantly solves the parsing problem
4. **Keeping Blake3 hashes simple** avoids IPFS complexity while maintaining clarity through context

The final decision was to stick with the current memo structure (separate headers and body) while adding a `kind` field for unambiguous parsing, balancing the needs of a general-purpose, self-describing format.