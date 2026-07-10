# TOOLING — The Oracle Pipeline

What gets built between here and the Definition of Done. One pipeline, seven tools. Game launches happen only at mint time (left box); everything an agent loop runs is offline (right side).

```
[real game + TraceLab mod]                        [any agent, sandboxed]
  scripts/trace_java.sh ──► data/traces/java/*.jsonl (frozen goldens, committed)
        ▲                            │
  data/traces/scripts/*.json ────────┼──► trace_replay bin ──► rust trace + diff
  (action scripts, committed)        │         │
                                     ▼         ▼
                              scripts/trace_diff.sh ──► logs/traces/<name>/report.json
                                               │                │
                              scripts/goal.sh coverage ──► ledger.json    ParityView (viz)
                              (goal.sh is future — U07; ledger.json is
                               seeded today by scripts/extract.sh)
```

## T1 — Trace schema (`packages/engine-rs/src/trace.rs`)

Serde structs, single source of truth, version field `v:1`. JSONL: record 0 is a header, then one record **after every executed action**.

```json
{"v":1,"kind":"header","seed":"3LGMWP6QYAWB","seed_long":57554006466,"character":"WATCHER","ascension":0,"game_version":"desktop-1.0","mods":["basemod","stslib","tracelab"]}
{"v":1,"idx":57,"floor":4,"turn":3,"phase":"COMBAT",
 "action":{"type":"PLAY_CARD","hand_idx":2,"card_id":"EmptyBody","target":0},
 "post":{
   "player":{"hp":61,"max_hp":72,"block":7,"energy":1,"stance":"CALM","gold":124,
             "powers":[{"id":"Vigor","amt":8}],"orbs":[]},
   "enemies":[{"id":"JawWorm","idx":0,"hp":30,"max_hp":44,"block":6,
               "intent":{"move_id":1,"name":"CHOMP","dmg":12,"hits":1},
               "powers":[],"move_history":[3,1]}],
   "piles":{"hand":["Strike_P","Defend_P","Eruption"],"draw_ordered":["..."],"discard":["..."],"exhaust":[]},
   "relics":[{"id":"PureWater","counter":-1}],"potions":["FairyPotion"],
   "rng":{"card":37,"ai":4,"shuffle":12,"cardRandom":0,"misc":2,"monsterHp":6,
          "treasure":3,"relic":11,"potion":5,"merchant":0,"eventList":1,"monster":8,"map":57}}}
```

RNG keys follow `docs/vault/rng-system-analysis.md` (13 streams) — vault names win over this example. Fixture test: Rust deserializes a committed Java-emitted trace (schema-drift tripwire).

## T2 — TraceLab mod (`packages/harness-java/`) — game-side

Maven project resurrected from git history (`git show ec608e30:mod/pom.xml`, EVTracker sources from `d71be8af`), package renamed `tracelab`, target `--release 8`.

- Seeded scripted launch (crib PracticeLab's launch patches); script path via `-Dtracelab.script=`, output via `-Dtracelab.out=`.
- Action feed waits for stable state (crib CommunicationMod's GameStateListener semantics — actionManager empty, no screen transition). Fallback if flaky: install CommunicationMod and drive it externally (protocol: `docs/vault/communication-mod-api.md`), accepting its weaker state dump to bootstrap.
- Trace writer = EVTracker's `TurnStateCapture` extended to per-action records + ordered piles + all 13 counters (public static fields on `AbstractDungeon`, no reflection).
- On script end/death: flush, exit (MTS `--close-when-finished`). Windowed is fine; never chase true headless.

**Action script** (`data/traces/scripts/<name>.json`) — the canonical action vocabulary, mapped 1:1 to `RunAction` on the Rust side:

```json
{"v":1,"seed":"3LGMWP6QYAWB","character":"WATCHER","ascension":0,"stop":{"max_floor":8},
 "actions":[{"type":"NEOW","choice":1},{"type":"PATH","choice":0},
            {"type":"PLAY_CARD","hand_idx":2,"target":0},{"type":"END_TURN"},
            {"type":"REWARD_TAKE","item":0},{"type":"REWARD_SKIP"},
            {"type":"EVENT_CHOICE","choice":1},{"type":"CAMPFIRE","choice":"REST"},
            {"type":"SHOP_BUY","item":2},{"type":"USE_POTION","idx":0,"target":0}]}
```

Entry: `scripts/trace_java.sh <script> <out>` — build-if-stale (mvn), copy jar to `$GAME/mods/`, launch via the proven `docs/vault/headless-launch.md` procedure. Model on existing `scripts/play.sh`.

## T3 — trace_replay + differ (`packages/engine-rs/src/bin/trace_replay.rs`)

First `[[bin]]` target (zero impact on lib tests). `trace_replay --script s.json --java-trace t.jsonl [--out rust.jsonl] --diff report.json --masks docs/goal/masks.json`.

Maps script actions → `RunAction`, steps `RunEngine`, emits the same schema, diffs field-by-field in canonical order (**rng counters first** — for enemy-AI work the counter delta is the diagnosis). Exit 0 ⇔ `"status":"match"` (masked-only diffs still exit 0 but list `masked`).

```json
{"status":"diverged","script":"act1-jawworm-3turn","seed":"3LGMWP6QYAWB",
 "matched_actions":56,"total_actions":112,
 "first_divergence":{"idx":57,"floor":4,"turn":3,
   "path":"post.enemies[0].intent.move_id","java":2,"rust":1,
   "rng_at_divergence":{"java":{"ai":9},"rust":{"ai":6}}},
 "masked":[],"secondary":[{"path":"post.rng.ai","java":9,"rust":6}]}
```

**Masks** (`docs/goal/masks.json`) — every entry MUST reference a `DEV-NNN` in the deviations register, scoped tight:

```json
[{"id":"DEV-003","path":"post.neow.options","scope":"all","reason":"engine intentionally exposes 4 Neow options","register":"docs/work_units/parity-deviations-register.md"}]
```

## T4 — `scripts/trace_diff.sh <script.json>`

Orchestrator: golden exists at `data/traces/java/<script-stem>.jsonl` → run trace_replay + differ; report + both traces land in `logs/traces/<script-stem>/`. Golden missing → fail with "needs mint", exit 3 (never auto-launches the game). Reuses `test_engine_rs.sh` env setup for cargo.

## T5 — Corpus + oracle tests

- Mint session (human-attended): A/B same-seed Java-vs-Java determinism check first, then ~10 A0 seeds (chosen from `$GAME/runs/WATCHER/`'s 311 runs + `docs/vault/verified-seeds.md` for coverage) + golden-run `1776347657` reconstruction. Goldens committed to `data/traces/java/` (protected).
- `src/tests/test_trace_oracle.rs` — replays every committed golden in-process, no game, part of the lib suite.

## T6 — `scripts/goal.sh` (bash + jq, the loop's steering wheel) — **future, not yet built (U03/U07)**

Today the ledger is seeded/refreshed by `scripts/extract.sh` (statuses preserved on re-run) and worked by hand per `AGENTS.md`; `goal.sh` adds tooling on top:

- `status` — units table + ledger counts (verified/unverified/quarantined) + next actionable item
- `next` — prints the single next thing to do (ready unit, else first unverified ledger row of the open unit)
- `coverage` — scan corpus traces, stamp `covered_by` on ledger rows, flag reachable-but-never-exercised
- `check-arch` — dependency-direction lint: core modules must not import `obs|search|training_contract|pyo3` (grep-based; DoD item 4)
- `quarantine <ledger-id> --dev DEV-NNN` — flips status + verifies register entry + mask exist

Ledger row (actual shape, `docs/goal/ledger.json`):

```json
{"id":"card/EmptyBody","kind":"card","class":"EmptyBody",
 "java_ref":"decompiled/java-src/com/megacrit/cardcrawl/cards/purple/EmptyBody.java",
 "methods_ref":"reference/extracted/methods/card/EmptyBody.java",
 "status":"unverified","verified_by":null,"dev":null}
```

Statuses: `unverified` → `verified` (source citation + source-derived test) or `quarantined` (DEV-NNN). U07 may add fields (e.g. `covered_by`) without breaking existing rows.

## T7 — ParityView (viz)

New view in the rescued `packages/viz`: pick a `logs/traces/*/report.json` → side-by-side Java|Rust panes at the divergence index, SVG sprites (`sprites/index.tsx`: Watcher w/ stance glow, enemies, intents), action scrubber, diverged fields highlighted from the report's paths. File-based (serve `logs/traces/` via existing viz server) — no WebSocket work.
