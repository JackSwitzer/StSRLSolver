#!/bin/bash
# Build mod and launch game

PROJECT_DIR="/Users/jackswitzer/Desktop/SlayTheSpireRL"

# Build
"$PROJECT_DIR/scripts/build.sh"
if [ $? -ne 0 ]; then
    exit 1
fi

# Launch
"$PROJECT_DIR/scripts/launch.sh" "$@"
