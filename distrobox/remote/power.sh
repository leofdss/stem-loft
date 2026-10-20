#!/usr/bin/env bash
# Start/stop what this project runs on the remote build host, so nothing
# holds its CPU, RAM or GPU while nobody is working (the host is usually
# someone's own machine, used for other things too). Rule: whoever
# finishes a work session on the host -- a person or an AI agent -- runs
# `power.sh down` (AGENTS.md, "Working in parallel").
#
#   distrobox/remote/power.sh up [ollama|build|all]     # default: all
#   distrobox/remote/power.sh down [ollama|build|all]   # default: all
#   distrobox/remote/power.sh status
#
#   ollama -- the stemloft-ollama service (local models, GPU)
#   build  -- the `stemloft` distrobox container (cargo/npm via run.sh)
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=/dev/null
[[ -f "$DIR/host.local" ]] && source "$DIR/host.local"
: "${STEMLOFT_REMOTE_HOST:?Set STEMLOFT_REMOTE_HOST (see distrobox/remote/host.local.example)}"

action="${1:-status}"
target="${2:-all}"
remote() { ssh -o BatchMode=yes -o ConnectTimeout=5 "$STEMLOFT_REMOTE_HOST" "$@"; }

ollama_up() {
  remote 'systemctl --user start stemloft-ollama.service'
  if [[ -n "${STEMLOFT_OLLAMA_URL:-}" ]]; then
    for _ in $(seq 1 60); do
      curl -fsS "$STEMLOFT_OLLAMA_URL/api/version" >/dev/null 2>&1 && return 0
      sleep 1
    done
    echo "ollama didn't answer at $STEMLOFT_OLLAMA_URL within 60s" >&2
    return 1
  fi
}
ollama_down() { remote 'systemctl --user stop stemloft-ollama.service' || true; }
build_up() { remote 'podman start stemloft >/dev/null'; }
build_down() { remote 'podman stop -t 10 stemloft >/dev/null 2>&1' || true; }

status() {
  remote 'printf "ollama: %s\n" "$(systemctl --user is-active stemloft-ollama.service 2>/dev/null)";
          printf "build:  %s\n" "$(podman inspect -f "{{.State.Status}}" stemloft 2>/dev/null || echo missing)";
          printf "gpu vram used: %s MiB\n" "$(( $(cat /sys/class/drm/card*/device/mem_info_vram_used 2>/dev/null | head -1) / 1048576 ))"'
}

case "$action:$target" in
  up:ollama) ollama_up ;;
  up:build) build_up ;;
  up:all) build_up; ollama_up ;;
  down:ollama) ollama_down ;;
  down:build) build_down ;;
  down:all) ollama_down; build_down ;;
  status:*) status ;;
  *) echo "usage: $0 up|down|status [ollama|build|all]" >&2; exit 2 ;;
esac
[[ "$action" == status ]] || status
