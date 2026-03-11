#!/bin/bash
# Launch the native Tauri app (requires Rust + tauri-cli)
# Falls back to browser if Tauri not available
# Usage: ./scripts/app.sh [--train]

set -e
cd "$(dirname "$0")/.."

CARGO_TAURI="/Users/jackswitzer/.cargo/bin/cargo-tauri"

if [ -f "$CARGO_TAURI" ] || command -v cargo-tauri &>/dev/null; then
    echo "Launching native Tauri app..."
    # Kill stale ports
    lsof -ti:8080 | xargs kill 2>/dev/null || true
    lsof -ti:5174 | xargs kill 2>/dev/null || true
    sleep 1

    # Start WS server in background
    uv run python -m packages.server --port 8080 &
    sleep 2

    # Launch Tauri dev mode (builds + opens native window)
    cd src-tauri
    /Users/jackswitzer/.cargo/bin/cargo tauri dev
else
    echo "Tauri CLI not found. Falling back to browser..."
    echo "Install: /Users/jackswitzer/.cargo/bin/cargo install tauri-cli"
    echo ""
    exec ./scripts/dev.sh "$@"
fi
