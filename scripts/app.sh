#!/bin/bash
# Launch the native macOS dashboard app (WKWebView → Vite dev server).
# Services (WS + Vite) are started before the app opens.
#
# Usage:
#   ./scripts/app.sh          # Start services + launch app (dev)
#   ./scripts/app.sh --build  # Build release .app bundle
#   ./scripts/app.sh --stop   # Stop all services
#   ./scripts/app.sh --status # Check service status

set -e
cd "$(dirname "$0")/.."

PROJECT_DIR="packages/viz/macos/STSTraining"
SCHEME="STSTraining"
BUILD_DIR="$PROJECT_DIR/build"

case "${1:-}" in
    --stop)
        ./scripts/services.sh stop
        exit 0
        ;;
    --build)
        echo "Building release app..."
        xcodebuild -project "$PROJECT_DIR/STSTraining.xcodeproj" \
            -scheme "$SCHEME" \
            -configuration Release \
            -derivedDataPath "$BUILD_DIR" \
            build 2>&1 | tail -5
        APP_PATH="$BUILD_DIR/Build/Products/Release/STSTraining.app"
        echo ""
        echo "Built: $APP_PATH"
        echo "Run:   open \"$APP_PATH\""
        exit 0
        ;;
    --status)
        ./scripts/services.sh status
        exit 0
        ;;
esac

# Clean shutdown on exit
trap './scripts/services.sh stop 2>/dev/null' EXIT INT TERM

echo "Launching STS Training Dashboard..."
echo ""

# Start WS server + Vite dev server
./scripts/services.sh start

# Wait for Vite to be ready
echo "Waiting for Vite dev server..."
for i in $(seq 1 30); do
    if curl -s -o /dev/null http://localhost:5174 2>/dev/null; then
        echo "Vite ready."
        break
    fi
    sleep 0.5
done

# Build and run debug app
echo "Building native app..."
xcodebuild -project "$PROJECT_DIR/STSTraining.xcodeproj" \
    -scheme "$SCHEME" \
    -configuration Debug \
    -derivedDataPath "$BUILD_DIR" \
    build 2>&1 | tail -3

APP_PATH="$BUILD_DIR/Build/Products/Debug/STSTraining.app"
echo "Opening $APP_PATH"
open "$APP_PATH"

# Keep script alive so trap fires on Ctrl-C
echo ""
echo "Press Ctrl-C to stop services and quit."
wait 2>/dev/null || cat
