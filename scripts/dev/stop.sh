#!/bin/bash
# Stop the STS dashboard server
set -e

echo "Stopping STS dashboard..."

# Stop via sts.py
cd "$(dirname "$0")/../.."
uv run scripts/sts.py stop 2>/dev/null || true

# Also kill any processes on port 8080
lsof -ti:8080 | xargs kill -9 2>/dev/null || true

echo "Dashboard stopped."
