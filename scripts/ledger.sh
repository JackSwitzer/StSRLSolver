#!/usr/bin/env bash
# Flip a verification-ledger row without hand-editing 667-row JSON.
# Usage:
#   scripts/ledger.sh flip <row-id> <verified_by>              # -> verified
#   scripts/ledger.sh quarantine <row-id> <verified_by> DEV-NNN
#   scripts/ledger.sh status                                    # counts + any unverified rows
set -euo pipefail
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LEDGER="$REPO/docs/goal/ledger.json"
cd "$REPO"

case "${1:-}" in
  flip)
    uv run python scripts/extract_content.py --ledger "$LEDGER" --flip "$2" --by "$3" --status verified
    ;;
  quarantine)
    uv run python scripts/extract_content.py --ledger "$LEDGER" --flip "$2" --by "$3" --status quarantined --dev "$4"
    ;;
  status)
    python3 - "$LEDGER" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
print("status_counts:", d["status_counts"])
nxt = [r["id"] for r in d["rows"] if r["status"] == "unverified"][:10]
print("next unverified:", nxt)
PY
    ;;
  *)
    echo "usage: $0 flip <row-id> <verified_by> | quarantine <row-id> <verified_by> DEV-NNN | status" >&2
    exit 2
    ;;
esac
