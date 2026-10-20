#!/usr/bin/env bash
# The full verification pass -- the single definition of "green" for this
# repo. A human runs exactly this before opening a PR, and so
# should any AI agent before calling a task done. Nothing here depends on
# which tool (if any) wrote the change.
#
#   scripts/verify.sh           # everything
#   scripts/verify.sh --quick   # repo/docs/OpenSpec checks only (no cargo/npm)
#
# Runs every step even after a failure, then exits 1 if any failed, so one
# run shows every problem at once.
set -uo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."
QUICK=0
[[ "${1:-}" == "--quick" ]] && QUICK=1

failed=()
step() {
  local name="$1"
  shift
  printf '\n== %s\n' "$name"
  if "$@"; then
    printf -- '-- ok: %s\n' "$name"
  else
    printf -- '-- FAILED: %s\n' "$name"
    failed+=("$name")
  fi
}

step "script unit tests" python3 -m unittest discover -s scripts/tests
step "agent-agnostic rule" python3 scripts/check_agent_agnostic.py
step "internal doc links" python3 scripts/check_doc_links.py
step "bilingual parity" python3 scripts/check_bilingual_parity.py
step "npm dependency baseline" python3 scripts/check_npm_deps.py ui/package.json
step "OpenSpec validation" scripts/openspec.sh validate --all --strict --no-interactive
step "kanban up to date" python3 scripts/openspec_to_kanban.py --check

if [[ $QUICK -eq 0 ]]; then
  step "cargo check" bash -c 'cd src-tauri && cargo check --all-targets'
  step "cargo clippy" bash -c 'cd src-tauri && cargo clippy --all-targets'
  step "cargo test" bash -c 'cd src-tauri && cargo test'
  # A fresh checkout installs from the lockfile; a dev tree keeps its node_modules.
  step "npm ci" bash -c 'cd ui && { [[ -d node_modules ]] || npm ci --no-audit --no-fund; }'
  step "prettier --check" bash -c 'cd ui && npx prettier --check .'
  step "npm run build" bash -c 'cd ui && npm run build'
  step "npm test" bash -c 'cd ui && npm run test -- --watch=false'
fi

echo
if [[ ${#failed[@]} -gt 0 ]]; then
  printf 'verify: %d step(s) failed:\n' "${#failed[@]}"
  printf '  - %s\n' "${failed[@]}"
  exit 1
fi
echo "verify: all steps passed"
