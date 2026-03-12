#!/bin/bash
# Browser-based dev environment (no native app).
# Uses services.sh for clean process management.
#
# Usage:
#   ./scripts/dev.sh              # Viz + WS server only
#   ./scripts/dev.sh --train      # + training workers

set -e
cd "$(dirname "$0")/.."

TRAIN=false
for arg in "$@"; do
    [ "$arg" = "--train" ] && TRAIN=true
done

trap './scripts/services.sh stop 2>/dev/null' EXIT INT TERM

./scripts/services.sh start

if [ "$TRAIN" = true ]; then
    echo ""
    echo "Training mode not yet wired via services.sh."
    echo "Training auto-starts when the frontend connects to WS."
fi

echo ""
echo "Open: http://localhost:${STS_VIZ_PORT:-5174}"
echo "Press Ctrl+C to stop."
wait
