#!/usr/bin/env bash
# Run a command on the remote build host, inside its `stemloft` distrobox
# container, against the copy of the project synced there by sync.sh.
#
# Usage:
#   distrobox/remote/run.sh [--dir src-tauri|ui] <command...>
#
# Examples:
#   distrobox/remote/run.sh --dir src-tauri cargo check
#   distrobox/remote/run.sh --dir src-tauri cargo clippy --all-targets
#   distrobox/remote/run.sh --dir ui npm run build
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "$DIR/lib.sh"
: "${STEMLOFT_REMOTE_HOST:?Set STEMLOFT_REMOTE_HOST (see distrobox/remote/host.local.example)}"
: "${STEMLOFT_REMOTE_PATH:?Set STEMLOFT_REMOTE_PATH (see distrobox/remote/host.local.example)}"

PROJECT_DIR="."
if [[ "${1:-}" == "--dir" ]]; then
  PROJECT_DIR="$2"
  shift 2
fi

if [[ $# -eq 0 ]]; then
  echo "Usage: $0 [--dir src-tauri|ui] <command...>" >&2
  exit 1
fi

printf -v CMD_QUOTED '%q ' "$@"
REMOTE_PATH="$REMOTE_ROOT"
ssh "$STEMLOFT_REMOTE_HOST" \
  "distrobox enter stemloft -- bash -lc 'cd $REMOTE_PATH/$PROJECT_DIR && $CMD_QUOTED'"
