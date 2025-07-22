# SZDT CLI

**S**igned **Z**ero-trust **D**a**T**a. Pronounced "Samizdat".

TDLR: signed CBOR for censorship-resistant data.

- [Whitepaper](./WHITEPAPER.md)
- [Website](https://szdt.dev)

The SZDT CLI provides command-line tools for signing and verifying data with SZDT.

## Quickstart

Install:

```bash
cargo install szdt_cli
```

Generate a keypair and give it a nickname ("alice"):

```bash
szdt key create alice
```

Create a data archive from a directory, signing it with your key:

```bash
szdt archive data/ --sign alice
```

Unarchive data:

```bash
szdt unarchive data.szdt
```

Signatures are verified during unpacking.

Check out `szdt --help` for more information.
