#!/bin/bash
# Watch game state changes - wrapper for watch_state.py
cd "$(dirname "$0")/../.."

# Pass all arguments to watch_state.py
uv run scripts/dev/watch_state.py "$@"
