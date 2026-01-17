#!/bin/bash
# Launch STS directly without Steam, auto-load mods

GAME_DIR="/Users/jackswitzer/Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources"
PROJECT_DIR="/Users/jackswitzer/Desktop/SlayTheSpireRL"
MODS_DIR="$GAME_DIR/mods"

# Ensure mods directory exists
mkdir -p "$MODS_DIR"

# Copy EVTracker if built
if [ -f "$PROJECT_DIR/mod/target/EVTracker.jar" ]; then
    cp "$PROJECT_DIR/mod/target/EVTracker.jar" "$MODS_DIR/"
fi

cd "$GAME_DIR"

# Launch with auto-mod loading (skip launcher, skip intro)
# Mods: basemod (required), evtracker (our logging)
exec ./jre/bin/java -Xmx1G -Xms512m \
    -jar ModTheSpire.jar \
    --skip-launcher \
    --skip-intro \
    --mods basemod,evtracker
