#!/bin/bash
# Parity test - compare Python predictions vs game
cd "$(dirname "$0")/../.."

# If no args, compare with current save
# If seed provided, generate predictions
uv run scripts/dev/test_parity.py "$@"
