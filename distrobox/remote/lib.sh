# Shared setup for sync.sh, run.sh, check.sh and power.sh -- sourced, not run.
#
# Sets:
#   LOCAL_ROOT   the checkout these scripts live in (main clone or a linked
#                git worktree)
#   REMOTE_ROOT  where that checkout lives on the remote host
# and sources host.local (STEMLOFT_REMOTE_HOST, STEMLOFT_REMOTE_PATH, ...).
#
# Parallel work (AGENTS.md, "Working in parallel") uses one git worktree
# per change. Two things make that safe here:
#   - host.local is gitignored, so a new worktree doesn't have one: it's
#     read from the main checkout instead.
#   - each worktree syncs to its own remote directory,
#     <STEMLOFT_REMOTE_PATH>-worktrees/<worktree folder name>, so two
#     workers never overwrite each other's files or share a cargo lock.

_REMOTE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOCAL_ROOT="$(cd "$_REMOTE_DIR/../.." && pwd)"

_git_dir="$(git -C "$LOCAL_ROOT" rev-parse --path-format=absolute --git-dir 2>/dev/null || true)"
_git_common="$(git -C "$LOCAL_ROOT" rev-parse --path-format=absolute --git-common-dir 2>/dev/null || true)"
STEMLOFT_IS_WORKTREE=0
if [[ -n "$_git_dir" && "$_git_dir" != "$_git_common" ]]; then
  STEMLOFT_IS_WORKTREE=1
fi

if [[ -f "$_REMOTE_DIR/host.local" ]]; then
  # shellcheck source=/dev/null
  source "$_REMOTE_DIR/host.local"
elif [[ $STEMLOFT_IS_WORKTREE == 1 && -f "$(dirname "$_git_common")/distrobox/remote/host.local" ]]; then
  # shellcheck source=/dev/null
  source "$(dirname "$_git_common")/distrobox/remote/host.local"
fi

if [[ -n "${STEMLOFT_REMOTE_PATH:-}" ]]; then
  STEMLOFT_MAIN_REMOTE_PATH="${STEMLOFT_REMOTE_PATH%/}"
  if [[ $STEMLOFT_IS_WORKTREE == 1 ]]; then
    REMOTE_ROOT="$STEMLOFT_MAIN_REMOTE_PATH-worktrees/$(basename "$LOCAL_ROOT")"
  else
    REMOTE_ROOT="$STEMLOFT_MAIN_REMOTE_PATH"
  fi
fi
