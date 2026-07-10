# harness-java — TraceLab

ModTheSpire mod that runs **seeded, scripted** Slay the Spire runs and dumps a
per-action JSONL state trace (all 13 RNG counters, ordered piles, intents). The
game-side half of the parity oracle (`docs/goal/TOOLING.md` T2, benchmark B0).

## Run

```bash
scripts/trace_java.sh <script.json> <out.jsonl>     # build + launch + collect
scripts/trace_java.sh <script.json> <out.jsonl> --no-build
```

Requires the real game install and launches a window (macOS). Offline replay/diff
against the emitted goldens is `scripts/trace_diff.sh` (no game needed).

## How it drives the game (the non-obvious incantations)

- **Launch**: `-Djava.awt.headless=false` is mandatory on macOS 26.1 or LWJGL2
  window creation segfaults silently. Never pass `--close-when-finished` with
  `-D` flags (child JVM drops them). See `docs/vault/headless-launch.md`.
- **Events / Neow** (`ScriptRunner.pressEventOption`): set `RoomEventDialog.selectedOption`
  + `waitForInput=false`; the event's next `update()` fires `buttonEffect`. Gate every
  press on `waitForInput==true` (dialog idle) or you clobber the prior selection.
  Single-option screens (Neow intro filler) auto-advance and are not recorded.
- **Map** (`ScriptRunner.choosePath`): the hover+`clicked` input path is unreachable
  (`DungeonMapScreen.updateMouse` resets `clicked` before nodes read it). Trigger the
  node's private `animWaitTimer` via reflection; `MapRoomNode.update` then runs the real
  `setCurrMapNode`/`nextRoomTransitionStart` path.
- **Combat**: standard `actionManager.cardQueue` + `PressEndTurnButtonAction`.

## Provenance

Evolved from EVTracker (`git show d71be8af -- mod/`, TurnStateCapture) and
PracticeLab (`git show ec608e30 -- mod/`, seeded launch). Package `tracelab`.
Build reference mods aren't committed; recover from history if needed.

## Status

Proven end-to-end: Neow → path → combat → END_TURN, byte-identical across runs
(benchmark B0). Remaining before full corpus (B2+): reward/campfire/shop/event-choice
executors for full-run scripts, and matching those actions to Rust `RunAction`.
