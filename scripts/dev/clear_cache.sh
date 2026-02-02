#!/bin/bash
# Clear Python bytecode cache for clean module reloading
set -e

cd "$(dirname "$0")/../.."
PROJECT_ROOT=$(pwd)

echo "Clearing Python cache..."

# Remove __pycache__ directories
find "$PROJECT_ROOT" -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true

# Remove .pyc files
find "$PROJECT_ROOT" -type f -name "*.pyc" -delete 2>/dev/null || true

# Remove .pyo files
find "$PROJECT_ROOT" -type f -name "*.pyo" -delete 2>/dev/null || true

echo "Cache cleared."
