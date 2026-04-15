#!/usr/bin/env bash
# Launch the Slay the Spire web play GUI.
set -euo pipefail
cd "$(dirname "$0")/.."

# Ensure uv is on PATH
export PATH="$HOME/.local/bin:$PATH"

echo "Starting Slay the Spire Play GUI at http://localhost:8421"
echo "Press Ctrl+C to stop."
PYTHONPATH="$(pwd)" uv run uvicorn packages.play.server:app --host 0.0.0.0 --port 8421
