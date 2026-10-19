#!/usr/bin/env bash
# Install (or update) the Ollama local-model server on the remote build
# host, as a rootless Podman Quadlet service, and pull the models
# distrobox/remote/ollama/models.toml marks `pull = true`.
#
# Run from your own machine; everything happens over SSH using
# distrobox/remote/host.local, like sync.sh/run.sh. Idempotent.
#
#   distrobox/remote/ollama/install.sh            # install/update + pull
#   distrobox/remote/ollama/install.sh --no-pull  # unit only
#
# The API listens only on the host's Tailscale IPv4 (override with
# OLLAMA_BIND_ADDR=...), never on every interface.
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=/dev/null
[[ -f "$DIR/../host.local" ]] && source "$DIR/../host.local"
: "${STEMLOFT_REMOTE_HOST:?Set STEMLOFT_REMOTE_HOST (see distrobox/remote/host.local.example)}"

PULL=1
[[ "${1:-}" == "--no-pull" ]] && PULL=0

BIND_ADDR="${OLLAMA_BIND_ADDR:-$(ssh "$STEMLOFT_REMOTE_HOST" 'tailscale ip -4 2>/dev/null | head -1')}"
if [[ -z "$BIND_ADDR" ]]; then
  echo "Couldn't find the host's Tailscale IPv4; set OLLAMA_BIND_ADDR explicitly." >&2
  exit 1
fi

echo "== Installing the Quadlet unit (API on $BIND_ADDR:11434)"
sed "s/@BIND_ADDR@/$BIND_ADDR/" "$DIR/ollama.container.in" |
  ssh "$STEMLOFT_REMOTE_HOST" 'mkdir -p ~/.config/containers/systemd && cat > ~/.config/containers/systemd/stemloft-ollama.container'
ssh "$STEMLOFT_REMOTE_HOST" 'systemctl --user daemon-reload && systemctl --user restart stemloft-ollama.service'

# Keep the service up while nobody is logged in. Needs no root for your own
# user on most distros; if polkit refuses, say so instead of failing.
if ! ssh "$STEMLOFT_REMOTE_HOST" 'loginctl show-user "$USER" -p Linger | grep -q yes || loginctl enable-linger "$USER"' 2>/dev/null; then
  echo "note: couldn't enable lingering -- run 'sudo loginctl enable-linger \$USER' on the host," >&2
  echo "      or the service stops when you log out there." >&2
fi

echo "== Waiting for the API"
for _ in $(seq 1 60); do
  curl -fsS "http://$BIND_ADDR:11434/api/version" >/dev/null 2>&1 && break
  sleep 2
done
curl -fsS "http://$BIND_ADDR:11434/api/version"
echo

if [[ $PULL -eq 1 ]]; then
  for model in $(python3 "$DIR/../../../scripts/local_llm.py" --list-pull); do
    echo "== Pulling $model"
    ssh "$STEMLOFT_REMOTE_HOST" "podman exec stemloft-ollama ollama pull $model"
  done
fi

echo
echo "Done. Point local tooling at it with, in distrobox/remote/host.local:"
echo "  export STEMLOFT_OLLAMA_URL=http://$BIND_ADDR:11434"
