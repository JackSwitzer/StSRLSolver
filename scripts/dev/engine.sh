#!/bin/bash
# Interactive Python Engine Tester
cd "$(dirname "$0")/../.."

SEED="${1:-1234567890}"
echo "Starting Python Engine with seed: $SEED"
echo "Compare with STS using the same seed."
echo ""

uv run tools/test_engine.py --seed "$SEED" --ascension 20
