#!/bin/bash
# Launch the native Tauri app.
# Services (WS + Vite) are managed by beforeDevCommand in tauri.conf.json
# which calls ./scripts/services.sh start.
#
# Usage:
#   ./scripts/app.sh          # Launch native app (dev mode)
#   ./scripts/app.sh --build  # Build production .app bundle
#   ./scripts/app.sh --stop   # Stop all services

set -e
cd "$(dirname "$0")/.."

case "${1:-}" in
    --stop)
        ./scripts/services.sh stop
        exit 0
        ;;
    --build)
        echo "Building production app..."
        PATH="$HOME/.cargo/bin:$PATH" cargo tauri build
        exit 0
        ;;
    --status)
        ./scripts/services.sh status
        exit 0
        ;;
esac

# Clean shutdown on exit
trap './scripts/services.sh stop 2>/dev/null' EXIT INT TERM

echo "Launching STS RL Mission Control..."
echo ""

# cargo tauri dev will:
#   1. Run beforeDevCommand (./scripts/services.sh start)
#   2. Wait for devUrl (http://localhost:5174)
#   3. Open native WebKit window
PATH="$HOME/.cargo/bin:$PATH" cargo tauri dev
