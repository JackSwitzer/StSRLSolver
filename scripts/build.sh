#!/bin/bash
# Build the EVTracker mod

PROJECT_DIR="/Users/jackswitzer/Desktop/SlayTheSpireRL"
cd "$PROJECT_DIR/mod"

echo "Building EVTracker mod..."
mvn clean package -q

if [ $? -eq 0 ]; then
    echo "Build successful: $PROJECT_DIR/mod/target/EVTracker.jar"
else
    echo "Build failed"
    exit 1
fi
