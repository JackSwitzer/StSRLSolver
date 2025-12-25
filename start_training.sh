#!/bin/bash
# Quick start script for overnight training

set -e

echo "=== STS AI Training Setup ==="

# Check for data
DATA_DIR="data/raw_2020"
if [ ! -d "$DATA_DIR" ] || [ -z "$(ls -A $DATA_DIR 2>/dev/null)" ]; then
    echo "ERROR: No training data found in $DATA_DIR"
    echo ""
    echo "To get training data:"
    echo "1. Go to: https://drive.google.com/drive/folders/1c7MwTdLxnPgvmPbBEfNWa45YAUU53H0l"
    echo "2. Download 2020 files (July-November for Watcher data)"
    echo "3. Place .json.gz files in: $DATA_DIR"
    echo ""
    exit 1
fi

echo "Found training data in $DATA_DIR"

# Process data
echo ""
echo "Step 1: Processing Watcher wins..."
uv run python3 data/process_watcher_data.py

# Check for processed data
PROCESSED_DATA="data/watcher_training/watcher_a20_wins.json"
if [ ! -f "$PROCESSED_DATA" ]; then
    echo "ERROR: Data processing failed"
    exit 1
fi

# Train model
echo ""
echo "Step 2: Training BC model..."
echo "This will run overnight. Check checkpoints/ for progress."
echo ""

# Use MPS on Mac, CUDA if available, else CPU
if python3 -c "import torch; print(torch.backends.mps.is_available())" 2>/dev/null | grep -q True; then
    DEVICE="mps"
elif python3 -c "import torch; print(torch.cuda.is_available())" 2>/dev/null | grep -q True; then
    DEVICE="cuda"
else
    DEVICE="cpu"
fi

echo "Using device: $DEVICE"

uv run python3 train_bc.py \
    --data "$PROCESSED_DATA" \
    --epochs 200 \
    --batch-size 128 \
    --device "$DEVICE"

echo ""
echo "=== Training Complete ==="
echo "Best model saved to: checkpoints/*/best_model.pt"
