#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
VIZ_DIR="$PROJECT_ROOT/packages/viz"

case "${1:-dev}" in
  dev)
    echo "Starting Spire Training Viewer (dev mode)..."
    echo "  API: http://localhost:8420"
    echo "  App: http://localhost:5173"

    # Start FastAPI in background
    cd "$VIZ_DIR"
    uv run uvicorn server:app --host 0.0.0.0 --port 8420 --reload &
    API_PID=$!

    # Start Vite dev server
    bun run dev &
    VITE_PID=$!

    trap "kill $API_PID $VITE_PID 2>/dev/null" EXIT
    wait
    ;;

  build)
    echo "Building frontend..."
    cd "$VIZ_DIR"
    bun run build
    echo "Done. Run 'scripts/viz.sh serve' to start."
    ;;

  serve)
    echo "Starting Spire Training Viewer (production)..."
    echo "  http://localhost:8420"
    cd "$VIZ_DIR"
    uv run uvicorn server:app --host 0.0.0.0 --port 8420
    ;;

  *)
    echo "Usage: scripts/viz.sh [dev|build|serve]"
    exit 1
    ;;
esac
