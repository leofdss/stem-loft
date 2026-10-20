#!/usr/bin/env bash
# One-shot remote validation pass: sync the working tree to the remote build
# host, then run scripts/verify.sh -- the repo's full verification pass -- on its
# CPU instead of the local machine's.
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
echo "== scripts/verify.sh on the remote host"
"$DIR/run.sh" scripts/verify.sh
