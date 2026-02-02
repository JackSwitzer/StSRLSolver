#!/bin/bash
# Read current save file - wrapper for read_save.py
cd "$(dirname "$0")/../.."

# Default to WATCHER, or pass character as arg
uv run scripts/dev/read_save.py "${1:-WATCHER}"
