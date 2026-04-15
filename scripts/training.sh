#!/usr/bin/env zsh
set -euo pipefail

uv run python -m packages.training "$@"
