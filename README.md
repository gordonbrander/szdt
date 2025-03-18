## TODO

- [ ] Envelope
  - [x] Serialize/deserialize CBOR
  - [x] Serialize/deserialize body
  - [x] Signing / verification
      - [x] Ed25519
- [ ] Archive
  - [x] Serialize/deserialize CBOR
  - [ ] Inlined Files
  - [ ] Links
  - [ ] Validate checksums
  - [ ] Metadata
  - [ ] update URLs
- [ ] CLI
  - [x] Generate key
  - [x] Rountrip envelope + archive
  - [x] Rountrip signed envelope + archive
  - [ ] Choose archive file name
  - [ ] Create from folder + manifest

## Development

### Installing binaries on your path with Cargo

From the project directory:

```bash
cargo install --path .
```

This will install the binaries to `~/.cargo/bin`, which is usually added to your path by the Rust installer.
