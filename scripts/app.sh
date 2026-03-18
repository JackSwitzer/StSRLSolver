#!/bin/bash
# Launch the Spire Monitor native macOS app.
#
# Usage:
#   ./scripts/app.sh          # Build and run (debug)
#   ./scripts/app.sh --build  # Build release
#   ./scripts/app.sh --run    # Run without rebuilding

set -e
cd "$(dirname "$0")/.."

APP_DIR="packages/app"

case "${1:-}" in
    --build)
        echo "Building Spire Monitor (release)..."
        cd "$APP_DIR"
        swift build -c release 2>&1 | tail -5
        echo "Built: .build/release/SpireMonitor"
        exit 0
        ;;
    --run)
        cd "$APP_DIR"
        .build/debug/SpireMonitor &
        echo "Spire Monitor running (PID $!)"
        exit 0
        ;;
esac

# Default: build debug + run
echo "Building Spire Monitor..."
cd "$APP_DIR"
swift build 2>&1 | tail -3
echo "Launching..."
.build/debug/SpireMonitor &
APP_PID=$!
echo "Spire Monitor running (PID $APP_PID)"
echo "Press Ctrl-C to quit."
trap "kill $APP_PID 2>/dev/null" EXIT INT TERM
wait $APP_PID 2>/dev/null || true
