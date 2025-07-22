---
layout: index.liquid
title: Quickstart - SZDT
---

# SZDT quickstart

Install CLI:

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
