#!/usr/bin/env bash
# Push the local working tree to a remote build host, where it gets
# checked/built/tested on faster hardware. See distrobox/remote/README.md.
#
# Usage:
#   distrobox/remote/sync.sh          # one-shot push
#   distrobox/remote/sync.sh --watch  # keep pushing every 2s (Ctrl+C to stop)
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "$DIR/lib.sh"
: "${STEMLOFT_REMOTE_HOST:?Set STEMLOFT_REMOTE_HOST (see distrobox/remote/host.local.example)}"
: "${STEMLOFT_REMOTE_PATH:?Set STEMLOFT_REMOTE_PATH (see distrobox/remote/host.local.example)}"

REMOTE_PATH="$REMOTE_ROOT/"
LOCAL_PATH="$LOCAL_ROOT/"

# A worktree's first sync starts from a copy of the main checkout's build
# output (cargo target/, ui/node_modules), so it doesn't pay a cold build of
# the whole Tauri stack. Skipped once the worktree has its own.
seed_worktree() {
  [[ $STEMLOFT_IS_WORKTREE == 1 ]] || return 0
  # shellcheck disable=SC2029  # expanded locally on purpose
  ssh "$STEMLOFT_REMOTE_HOST" "
    mkdir -p '$REMOTE_ROOT/src-tauri' '$REMOTE_ROOT/ui'
    for d in src-tauri/target ui/node_modules; do
      if [ ! -e '$REMOTE_ROOT/'\$d ] && [ -d '$STEMLOFT_MAIN_REMOTE_PATH/'\$d ]; then
        cp -a --reflink=auto '$STEMLOFT_MAIN_REMOTE_PATH/'\$d '$REMOTE_ROOT/'\$d
      fi
    done"
}

do_sync() {
  # --mkpath: a fresh host (or a new STEMLOFT_REMOTE_PATH) has no target dir yet.
  rsync -az --delete --mkpath \
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
  seed_worktree
  do_sync
  echo "Synced -> $STEMLOFT_REMOTE_HOST:$REMOTE_PATH"
fi
