#!/usr/bin/env bash
# Regenerate the decompiled Java reference at decompiled/java-src (gitignored,
# local-only). Ground truth for all parity work: docs/goal/GOAL.md.
# Recipe recorded in decompiled/manifest.json; run this on any new machine.
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GAME="$HOME/Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources"
SRC_JAR="$GAME/desktop-1.0.jar"
SRC_SHA_EXPECTED="dd60a613a6178e08f1a57bcb8d5747e33c21fd49e743b88d76e9d74042803b2a"
CFR_JAR="$REPO/decompiled/.tools/cfr-0.152.jar"
CFR_SHA_EXPECTED="f686e8f3ded377d7bc87d216a90e9e9512df4156e75b06c655a16648ae8765b2"
CFR_URL="https://repo1.maven.org/maven2/org/benf/cfr/0.152/cfr-0.152.jar"
OUT="$REPO/decompiled/java-src"
FILTER="com/megacrit/cardcrawl"
JAVA_BIN="${DECOMPILE_JAVA:-$GAME/jre/bin/java}"

[[ -f "$SRC_JAR" ]] || { echo "game jar not found: $SRC_JAR" >&2; exit 2; }
SRC_SHA=$(shasum -a 256 "$SRC_JAR" | cut -d' ' -f1)
[[ "$SRC_SHA" == "$SRC_SHA_EXPECTED" ]] || echo "WARNING: desktop-1.0.jar sha256 differs from manifest (game updated?): $SRC_SHA" >&2

if [[ ! -f "$CFR_JAR" ]]; then
  echo "fetching CFR 0.152"
  mkdir -p "$(dirname "$CFR_JAR")"
  curl -fsSL "$CFR_URL" -o "$CFR_JAR"
fi
CFR_SHA=$(shasum -a 256 "$CFR_JAR" | cut -d' ' -f1)
[[ "$CFR_SHA" == "$CFR_SHA_EXPECTED" ]] || { echo "cfr jar sha mismatch: $CFR_SHA" >&2; exit 3; }

mkdir -p "$OUT"
echo "decompiling $FILTER/** from desktop-1.0.jar (takes a few minutes)"
"$JAVA_BIN" -Xmx2G -jar "$CFR_JAR" "$SRC_JAR" \
  --outputdir "$OUT" --jarfilter "^$FILTER" --silent true

COUNT=$(find "$OUT/$FILTER" -name '*.java' | wc -l | tr -d ' ')
python3 - "$REPO" "$SRC_SHA" "$CFR_SHA" "$COUNT" <<'PY' 2>/dev/null || true
import json, sys, datetime
repo, src_sha, cfr_sha, count = sys.argv[1:5]
json.dump({"source_jar_sha256": src_sha, "cfr_jar_sha256": cfr_sha,
           "filtered_class_count": int(count),
           "generated_at": datetime.datetime.now(datetime.timezone.utc).isoformat()},
          open(f"{repo}/decompiled/manifest.json", "w"), indent=2, sort_keys=True)
PY
echo "done: $COUNT java files under decompiled/java-src/$FILTER"
