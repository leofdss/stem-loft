#!/usr/bin/env bash
# One-shot remote validation pass: sync the working tree to the remote build
# host, then run the Rust core's check/clippy/test and the Angular UI's build
# on its CPU instead of the local machine's.
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

"$DIR/sync.sh"

echo
echo "== src-tauri: cargo check + clippy + test =="
"$DIR/run.sh" --dir src-tauri cargo check
"$DIR/run.sh" --dir src-tauri cargo clippy --all-targets
"$DIR/run.sh" --dir src-tauri cargo test

echo
echo "== ui: npm build =="
"$DIR/run.sh" --dir ui npm run build
