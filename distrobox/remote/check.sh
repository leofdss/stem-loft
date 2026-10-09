#!/usr/bin/env bash
# One-shot remote validation pass: sync the working tree to the remote build
# host, then run the Rust core's check/clippy/test and the Angular UI's
# build + test suite on its CPU instead of the local machine's.
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# The remote container must run the exact image the local one does --
# decided by scripts/check_remote_image.py comparing podman image IDs, not
# by anyone eyeballing tags/dates. Exit 1 (mismatch) stops here; exit 2
# (couldn't compare, e.g. no local podman) only warns.
set +e
python3 "$DIR/../../scripts/check_remote_image.py"
image_status=$?
set -e
if [[ $image_status -eq 1 ]]; then
  echo "Remote image differs from local -- fix with: python3 scripts/check_remote_image.py --fix" >&2
  exit 1
fi

"$DIR/sync.sh"

echo
echo "== src-tauri: cargo check + clippy + test =="
"$DIR/run.sh" --dir src-tauri cargo check
"$DIR/run.sh" --dir src-tauri cargo clippy --all-targets
"$DIR/run.sh" --dir src-tauri cargo test

echo
echo "== ui: npm build + test (coverage) =="
"$DIR/run.sh" --dir ui npm run build
"$DIR/run.sh" --dir ui npm run test -- --watch=false
