#!/usr/bin/env bash
# Regenerate reference/extracted/ (data tables + method index) from the
# decompiled Java, and refresh the committed verification ledger.
set -euo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO"
uv run python scripts/extract_content.py --ledger docs/goal/ledger.json "$@"
