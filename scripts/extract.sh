#!/usr/bin/env bash
# Regenerate reference/extracted/ (data tables + method index) from the
# decompiled Java, and merge-refresh the committed verification ledger without
# changing existing row statuses.
set -euo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO"

# Worktree convenience: decompiled/ is local-only (gitignored), so a fresh
# git worktree lacks it. If the main checkout has one, symlink it in rather
# than failing (decompile_java.sh takes minutes and needs the game install).
if [[ ! -e "$REPO/decompiled/java-src" ]]; then
  MAIN_CHECKOUT="$(cd "$(git rev-parse --git-common-dir)/.." && pwd)"
  if [[ "$MAIN_CHECKOUT" != "$REPO" && -d "$MAIN_CHECKOUT/decompiled/java-src" ]]; then
    echo "WARNING: decompiled/ missing here; symlinking from main checkout: $MAIN_CHECKOUT/decompiled" >&2
    ln -sfn "$MAIN_CHECKOUT/decompiled" "$REPO/decompiled"
  fi
fi

uv run python scripts/extract_content.py --ledger docs/goal/ledger.json "$@"
