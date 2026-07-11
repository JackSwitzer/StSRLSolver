#!/usr/bin/env bash
# Launch Slay the Spire with TraceLab passive recording: play normally (any
# character/seed/ascension picked in the game UI); every run you play is
# written as one golden trace file to data/traces/recordings/.
# Usage: scripts/play_record.sh   (window opens; play; quit whenever)
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GAME="$HOME/Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources"
HARNESS="$REPO/packages/harness-java"
OUT_DIR="$REPO/data/traces/recordings"
mkdir -p "$OUT_DIR"

if [[ ! -f "$HARNESS/target/TraceLab.jar" || -n "$(find "$HARNESS/src" -newer "$HARNESS/target/TraceLab.jar" -type f 2>/dev/null | head -1)" ]]; then
  echo "[play_record] building TraceLab.jar"
  JAVA_HOME="${TRACELAB_JAVA_HOME:-/opt/homebrew/opt/openjdk@11}" /opt/homebrew/bin/mvn -q -f "$HARNESS/pom.xml" clean package
fi
cp "$HARNESS/target/TraceLab.jar" "$GAME/mods/TraceLab.jar"

echo "[play_record] launching — traces land in $OUT_DIR (one file per run)"
cd "$GAME"
exec ./jre/bin/java -Xmx2G \
  -Djava.awt.headless=false \
  -Dtracelab.dir="$OUT_DIR" \
  -jar ModTheSpire.jar --mods basemod,tracelab --skip-intro
