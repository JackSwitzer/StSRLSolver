#!/bin/bash
# Launch Slay the Spire with mods for AI control - no GUI needed

GAME_DIR="/Users/jackswitzer/Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources"
WORKSHOP_MTS="/Users/jackswitzer/Library/Application Support/Steam/steamapps/workshop/content/646570/1605060445/ModTheSpire.jar"

echo "=== Slay the Spire AI Launcher ==="
echo "Auto-loading mods: basemod, stslib, CommunicationMod"
echo ""

# Use Workshop ModTheSpire if available, otherwise fall back to bundled
if [ -f "$WORKSHOP_MTS" ]; then
    MTS_JAR="$WORKSHOP_MTS"
    echo "Using Workshop ModTheSpire"
else
    MTS_JAR="$GAME_DIR/ModTheSpire.jar"
    echo "Using bundled ModTheSpire"
fi

cd "$GAME_DIR"

# Launch with auto-loaded mods, skip GUI and intro
./jre/bin/java -jar "$MTS_JAR" --mods basemod,stslib,CommunicationMod --skip-intro

echo ""
echo "Game exited."
