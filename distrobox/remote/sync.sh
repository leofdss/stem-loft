#!/usr/bin/env bash
# Push the local working tree to a remote build host, where it gets
# checked/built/tested on faster hardware. See distrobox/remote/README.md.
#
# Usage:
#   distrobox/remote/sync.sh          # one-shot push
#   distrobox/remote/sync.sh --watch  # keep pushing every 2s (Ctrl+C to stop)
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=/dev/null
[[ -f "$DIR/host.local" ]] && source "$DIR/host.local"
: "${STEMLOFT_REMOTE_HOST:?Set STEMLOFT_REMOTE_HOST (see distrobox/remote/host.local.example)}"
: "${STEMLOFT_REMOTE_PATH:?Set STEMLOFT_REMOTE_PATH (see distrobox/remote/host.local.example)}"

REMOTE_PATH="${STEMLOFT_REMOTE_PATH%/}/"
LOCAL_PATH="$(cd "$DIR/../.." && pwd)/"

do_sync() {
  rsync -az --delete \
    --exclude '.git' \
    --exclude 'target' \
    --exclude 'node_modules' \
    --exclude 'dist' \
    --exclude '.angular' \
    --exclude '*.log' \
    "$LOCAL_PATH" "$STEMLOFT_REMOTE_HOST:$REMOTE_PATH"
}

if [[ "${1:-}" == "--watch" ]]; then
  echo "Syncing $LOCAL_PATH -> $STEMLOFT_REMOTE_HOST:$REMOTE_PATH every 2s (Ctrl+C to stop)"
  while true; do
    do_sync
    sleep 2
  done
else
  do_sync
  echo "Synced -> $STEMLOFT_REMOTE_HOST:$REMOTE_PATH"
fi
