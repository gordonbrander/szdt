#!/bin/bash
set -e

wasm-pack build --target nodejs --out-dir pkg-node
