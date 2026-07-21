# SPEC: TraceLab Record-Mode

## One-liner
While you play Slay the Spire normally, the mod records every decision you make (the
action script) and the full game state after each one (the trace), as gzip JSONL,
flushed per action, resumable across sittings — data high-fidelity enough to exactly
recreate the run in the Rust sim later.

## Success Criteria
- [ ] A complete played run (Neow → death/victory/Heart) yields `<run-id>.script.json`
      + `<run-id>.trace.jsonl.gz` with one record per decision, no gaps.
- [ ] Every record carries: action (typed, v2 vocabulary), post-action state (HP, gold,
      energy, block, stance, orbs, ordered piles, hand, enemies with intents/powers/
      move history, relics WITH counters, potions, deck, map position, floor, act,
      screen/phase id), and all 13 RNG stream counters.
- [ ] Save-and-quit is logged as an explicit `SAVE_QUIT` event; relaunching and
      continuing the run appends to the same artifacts with no discontinuity.
- [ ] An offline completeness validator (`scripts/validate_recording.sh`) proves
      "sufficient to recreate": schema-valid, action per state transition, counters
      non-null and monotonic per stream semantics, no unknown-action records.
- [ ] Recording adds no perceptible input latency (flush is async or per-action tiny).
- [ ] Existing 6-action ScriptRunner and smoke-remint flow untouched and still working.

## User Flow
1. Launch modded game normally (record-mode is on whenever the mod is loaded).
2. Play. Every decision — Neow choice, path, card plays, targets, potions (use/discard),
   event choices, shop buys/removes, rewards taken/skipped, campfire actions, chests,
   boss relics, save-and-quit — is captured at the moment the game commits it.
3. Quit whenever; relaunch continues the run and the recording.
4. On run end, artifacts finalize in `data/traces/recordings/`. Run the validator; green
   recordings get committed. (Parity replay through the sim is a later phase — v1 only
   guarantees the data is complete enough.)

## Technical Decisions
- **Capture at commit points, not input events**: patch the same game methods
  CommunicationMod observes (its GameStateListener/patch set is the map — crib from
  `docs/vault/communication-mod-api.md` + its source, no runtime dependency on it).
  Hook where the game state machine commits a decision (e.g. card queued → played,
  reward claimed, event option resolved), so recorded actions are exactly the decisions
  the sim's `RunAction` vocabulary replays.
- **One writer, two artifacts**: extend the existing `TraceWriter` (already emits
  per-action records + 13 counters) rather than a parallel system; the script is derived
  from the same event stream. Add relic counters + screen/phase id + map coords to the
  record schema (Rust side already emits real relic counters as of wave 1).
- **gzip JSONL, comprehensive over compact**: full state every record; GZIPOutputStream
  with per-action flush (SYNC_FLUSH) so a kill loses at most the in-flight record.
- **Resume**: on `CardCrawlGame`/save-load of a continued run, locate artifacts by run-id
  (seed + character + start timestamp persisted in a sidecar meta json), reopen in append
  mode, log a `RESUME` event. Java persists RNG counters through save/load, so the trace
  stays continuous.
- **Action vocabulary = script schema v2** (`docs/work_units/script-schema-v2.md`, being
  produced by the Wave-2 engine work). The mod emits the same names/fields — the schema
  doc is the shared contract; any action the mod sees that v2 lacks gets recorded as
  `{"type":"UNKNOWN","raw":...}` and flagged by the validator (never silently dropped).
- **Storage**: `data/traces/recordings/<run-id>/` (NOT the protected `data/traces/java/`
  goldens dir; promotion to golden status is a deliberate later step).

## Data Model
- `meta.json` — run-id, seed (long + display string), character, ascension, mod/game
  versions, sittings (list of launch/quit timestamps).
- `script.json` — v2 action list as played (appended per action; the "what I did").
- `trace.jsonl.gz` — one record per action: `{idx, floor, act, turn, action, post:{...},
  rng:{13 counters}}` (the "what the game said happened").
- Lifecycle events interleaved in both: `RUN_START`, `SAVE_QUIT`, `RESUME`,
  `RUN_END{victory|death|abandon}`.

## Edge Cases
- Save-and-quit mid-combat: StS restores combat from save; `RESUME` record follows
  `SAVE_QUIT`; counters continue (game persists them).
- Run abandoned from menu: `RUN_END{abandon}`; artifacts finalized, validator accepts.
- Non-Watcher/non-A0 runs: recorded and tagged; parity scope filtering happens
  downstream, the recorder never discards.
- Grid/complex choices (Match & Keep, card selects, scry, Gambler's): each resolves to a
  typed action with indices at commit time; if ambiguous, `UNKNOWN` + validator flag
  rather than a guess.
- Multiple runs per launch: run-id rotates on dungeon generation; artifacts never mix.

## Out of Scope (v1)
- Full-vocabulary Java script EXECUTOR (existing 6-action runner stays as-is).
- Parity replay gate (v1 proves completeness, not sim-match; replay is the next phase
  once Wave 2's engine vocabulary lands).
- Delta/keyframe encoding, record-mode UI/overlay, other characters' parity, CI of the
  Maven build.

## Open Questions
- Exact hook-point list per action type — resolved during P1 by reading CommunicationMod
  patches + game source; the inventory doc is a P1 deliverable, reviewed before P2 code.

## Implementation Plan
1. **P1 Hook inventory**: enumerate every decision commit-point (Java class.method per
   v2 action) cribbing CommunicationMod; deliver `packages/harness-java/HOOKS.md`.
2. **P2 Recorder core**: RecordMode patches + extended TraceWriter (gzip, per-action
   flush, relic counters, screen id), script emission, run-id/meta lifecycle.
3. **P3 Resume**: sidecar meta, append-reopen, SAVE_QUIT/RESUME events.
4. **P4 Validator**: `scripts/validate_recording.sh` (bash + jq + zcat) — schema,
   continuity, counter sanity, UNKNOWN-action report.
5. **P5 Pilot**: you play one short A0 Watcher run; validator green; review record
   fidelity together before longer sessions.
