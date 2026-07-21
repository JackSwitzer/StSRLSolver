# TraceLab Mod — Data-Collection Scope (pre-interview draft)

Purpose: scope the remaining `packages/harness-java/` work so the /interview can settle
the open decisions and Codex can implement the rest without design churn. The engine-side
half of the contract is Wave 2 (`WAVE2_PROMPT.md` → `docs/work_units/script-schema-v2.md`);
the mod implements the SAME schema.

## What exists today (verified)

- `TraceLabMod.parseSeed` — decimal-first seed parsing, header `seed_long` written from the
  same call; contract confirmed correct by independent verification (2026-07-16).
- `ScriptRunner` executes only 6 action types: PLAY_CARD, END_TURN, USE_POTION, NEOW,
  EVENT_CHOICE, PATH. Everything else prints "unsupported action type" — full runs are
  not scriptable.
- `TraceWriter` — per-action records, ordered piles, all 13 RNG counters (public static
  fields, no reflection). Known nits (FINDINGS F7): `-1` for null RNG streams would break
  Rust deserialization on pre-run records; `choice` typed Integer vs Rust's string CAMPFIRE;
  no `max_actions` stop condition Java-side.
- `scripts/trace_java.sh` — build-if-stale, jar copy, attended launch (B0 proven, A/B
  byte-identical).

## Work items (Codex-implementable once decisions land)

- H1 — Action executors for the full v2 vocabulary. Each action needs its Java UI entry
  point and a stable-state wait condition: CAMPFIRE (CampfireUI options incl. RECALL),
  SHOP_BUY/SHOP_REMOVE/SHOP_LEAVE (ShopScreen purchase paths), REWARD_TAKE/REWARD_SKIP +
  CARD_REWARD (CombatRewardScreen / CardRewardScreen incl. Singing Bowl), CHEST_OPEN/SKIP,
  BOSS_RELIC pick/skip, DISCARD_POTION, PROCEED. Acceptance per action: scripted execution
  from a save state reaches the expected next stable state and the trace record round-trips
  through the Rust replayer.
- H2 — Pre-flight validation (VERIFY BEFORE HUMAN TIME — hard requirement). Before any
  attended session: (a) the Rust engine replays the script offline and reports the expected
  per-action phase sequence; (b) the mod gets a `--dry-run` script lint (schema-valid,
  actions in-vocabulary, stop condition present). A script that fails either never reaches
  a human session. Rationale: the project's error history is Java-misreading; burn CPU, not
  attended sessions.
- H3 — Batch minting. One attended launch drains a queue (`data/traces/requests/*.json`):
  auto-advance to next script on script-end/death, per-script A/B determinism re-run,
  output validation (13 non-null counters on every record, monotonic action idx), reject-
  and-continue on validation failure. Human is present but idle except for launch and
  anomalies.
- H4 — Crash/desync recovery for long scripts (Heart runs are hundreds of actions):
  periodic autosave alignment or restart-and-replay-to-floor semantics. Needs a decision
  (see Q4).
- H5 — F7 nits: emit omitted-vs-`-1` for null streams, CAMPFIRE choice type, `max_actions`
  parity with the Rust stop condition.

## Open questions for the /interview (user decisions)

- Q1 — Record shape & size. Full state per action is heavy for full runs. Options:
  (a) gzip JSONL as-is (~10x, zero schema change, differ untouched);
  (b) keyframe + delta records (floor-start keyframes, per-action diffs; smallest, but the
      differ and TraceWriter both change);
  (c) full records but piles-as-ids + string-table header (middle ground).
  Recommendation: (a) now — goldens are append-only artifacts and gzip is transparent to
  jq/differ via zcat; revisit (b) only if corpus size actually hurts.
- Q2 — What extra state to gather per record beyond the current schema (user "state
  gathering" ideas go here): candidate additions — screen/phase id, potion slots as typed
  list, map node coords, event id + choice text hash, relic counters (Rust side now emits
  real counters; Java side should match).
- Q3 — Action-definition granularity the user wants for "all actions I take": is a raw
  human-played session recordable as a script (record-mode: play normally, mod writes the
  action list + trace simultaneously), or are scripts always authored offline? Record-mode
  is the fastest path to reconstructing golden run 1776347657 and directly serves
  data collection beyond parity.
- Q4 — Recovery semantics for H4 (restart-replay vs save-scumming vs accept-loss).
- Q5 — Corpus definition: stick with ~10 coverage-maximizing A0 seeds + golden run, or
  extend now that the engine can brute-force seed selection offline?

## Session protocol (unchanged)

Attended sessions mint goldens; agents never launch the game. First session on the queue:
remint `smoke-neow-floor1` (same script, unchanged seed 57554006466) — verified expected
first combat: Small Slimes (SpikeSlime_S + AcidSlime_M). If it produces JawWorm again,
escalate instead of masking; three independent derivations say it cannot.
