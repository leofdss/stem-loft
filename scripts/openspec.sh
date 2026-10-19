#!/usr/bin/env bash
# Pinned OpenSpec CLI (https://github.com/Fission-AI/OpenSpec) -- the one
# place its version is decided. Runs through npx, so it's never a
# dependency of ui/ (see docs/architecture.md#stack-and-platform).
# Usage: scripts/openspec.sh <openspec args...>   e.g. scripts/openspec.sh list
# OPENSPEC_ROOT=<dir> runs it against another tree (scripts/openspec_archive.py
# uses that for the pt-BR shadow archive); default is this repo.
set -euo pipefail
OPENSPEC_VERSION="1.14.1"
export OPENSPEC_TELEMETRY="${OPENSPEC_TELEMETRY:-0}"
cd "${OPENSPEC_ROOT:-$(dirname "${BASH_SOURCE[0]}")/..}"
exec npx --yes --loglevel=error "@fission-ai/openspec@${OPENSPEC_VERSION}" "$@"
