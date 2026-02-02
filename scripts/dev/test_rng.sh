#!/bin/bash
# Test RNG predictions - wrapper for test_rng.py
cd "$(dirname "$0")/../.."

# Pass all arguments to test_rng.py
uv run scripts/dev/test_rng.py "$@"
