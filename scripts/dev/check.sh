#!/bin/bash
# Quick API state check - wrapper for check_api.py
cd "$(dirname "$0")/../.."

# Pass argument directly (keys, neow, bosses, rng, raw, diagnose)
# Default shows summary
uv run scripts/dev/check_api.py "$@"
