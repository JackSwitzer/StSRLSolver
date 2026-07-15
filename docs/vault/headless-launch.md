# Human TraceLab launch reference

Despite this filename, the proven macOS TraceLab path is not headless. It
launches a game window with java.awt.headless=false. This procedure is for a
human minting session only; agents must never launch the game.

## Supported entry point

From the repository root, a human uses:

    scripts/trace_java.sh <script.json> <out.jsonl>

or, when the existing TraceLab JAR is known current:

    scripts/trace_java.sh <script.json> <out.jsonl> --no-build

The script is the authority for current paths and flags. It builds
packages/harness-java, installs TraceLab.jar into the local game mods
directory, launches ModTheSpire with BaseMod and TraceLab, waits for exit, and
checks that a nonempty JSONL file was written.

## Current requirements

- a local Slay the Spire macOS installation at the path checked by the script;
- ModTheSpire and BaseMod installed for that game copy;
- Maven and the Java runtime configured by scripts/trace_java.sh;
- a TraceLab action script under data/traces/scripts/ or another explicit path;
- a human present to observe and stop a failed run.

The current launch keeps java.awt.headless=false. Do not add
--close-when-finished to the Java command: the proven path relies on JVM
properties reaching the game process.

## Offline work

No game launch is needed to compare a committed golden:

    scripts/trace_diff.sh data/traces/scripts/<script>.json

Generated logs and Rust replay output go under logs/traces/. Java goldens under
data/traces/java/ are protected and are minted only by the human workflow.

For implementation details, read packages/harness-java/README.md and the
current scripts rather than copying old launch commands from git history.
