#!/bin/sh
# Build the wasm version and serve it at http://localhost:8000.
set -e
cd "$(dirname "$0")/.."
cargo build --release --target wasm32-unknown-unknown
rm -rf target/web && mkdir -p target/web
cp web/* target/wasm32-unknown-unknown/release/ragdoll_cricket.wasm target/web/
python3 -m http.server 8000 -d target/web
