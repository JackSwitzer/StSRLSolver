# Comprehensive Parity Inconsistency Report

Last updated: 2026-04-14  
Branch: `codex/universal-gameplay-runtime`

This is the canonical parity audit for `packages/engine-rs`. It reconciles live source, focused suite results, intentional design decisions, and the remaining all-content tail.

## 1. Executive Summary

Current read:

- supported-scope runtime parity: `~99%`
- all-content gameplay parity: `~98%`
- supported-scope merge blockers: `0`
- all-content blockers still open before we can honestly claim “full gameplay content complete”: `3`

What is truly done:

- public gameplay-gap card tail: `0`
- raw public `complex_hook` tail: `0`
- supported event blocked-op tail: `0`
- Neow action layer is real and intentionally always exposes `4` choices
- potion legality / choose-one runtime path is landed
- `Emotion Chip` timing is landed
- `Scrap Ooze` is landed
- `NoteForYourself` now runs as a real cross-run card-stash flow inside the canonical event / reward runtime

What is still open:

- `Match and Keep!` still lacks the Java GremlinMatchGame minigame runtime
- `Liquid Memories` still lacks exact Java discard-choice selection
- `Smoke Bomb` still has one explicit positional caveat for Java `BackAttack` legality because the Rust combat model does not represent Surrounded/position state

Bottom line:

- If the claim is `supported runtime parity complete`, this branch is ready after cleanup/doc sync.
- If the claim is `all gameplay content complete`, that stronger claim is still false until section 3 is closed.

## 2. Quantified Baseline

### Inventory

| Metric | Current value | Notes |
| --- | ---: | --- |
| Registered card ids | `717` | Last verified registry-export baseline; raw source currently has `719` unique `id:` declarations |
| Typed event names | `52` | `62` `event(...)` call sites including continuation sub-states |
| Potion ids | `42` | Current source scan |
| Relic ids | `102` | Current source scan |
| Raw public gameplay-gap files | `0` | After excluding cleanup-only shells |
| Cleanup-only empty shells | `3` | `Reflex`, `Tactician`, `Deus Ex Machina` |
| Raw public `complex_hook` files | `0` | Current source scan |
| Blocked supported event ops | `0` | Current source scan |
| Explicit blocked event branches in source | `1` | `Match and Keep!` |
| Direct `#[ignore]` count in `src/tests` | `75` | Current source scan |

### Current status table

| Bucket | Current state |
| --- | --- |
| Fully supported | public card gameplay behavior, supported event runtime, Neow 4-choice action surface, Scrap Ooze, Emotion Chip timing, potion action path, reward/runtime ordering, RL/search surfaces |
| Cleanup-only shells | `Reflex`, `Tactician`, `Deus Ex Machina` |
| Explicit blocked / not yet closed | `Match and Keep!` minigame |
| Explicit semantic caveats | `Liquid Memories` discard-choice fidelity, `Smoke Bomb` back-attack positional legality |

### Ignored-test family summary

| Family | Current direct ignored count |
| --- | ---: |
| Watcher | `21` |
| Colorless / choice | `27` |
| Defect / orb | `8` |
| Ironclad | `1` |
| Potions | `1` |
| Other | `17` |

Some raw counts are intentionally noisy unless classified:

- the `3` raw empty public-card files are not gameplay gaps
- the `75` ignored tests include a mix of live blockers, stale solved lines, and cleanup-only noise
- `Match and Keep!` is the only remaining explicit blocked event branch in source

### Why we believe the engine works

These representative green suites were re-run on the current local tree during this audit pass:

| Area | Suite | Result |
| --- | --- | --- |
| Wrapper gate | `./scripts/test_engine_rs.sh check --lib` | green |
| Wrapper gate | `./scripts/test_engine_rs.sh test --lib --no-run` | green |
| Run / RL | `test_run_parity` | `19 passed` |
| Run / RL | `test_rl_contract` | `12 passed` |
| Search | `test_search_harness` | `5 passed` |
| Rewards | `test_reward_runtime` | `10 passed` |
| Events | `test_events_parity` | `7 passed` |
| Events | `test_event_runtime_wave19` | `3 passed` |
| Events | `test_event_runtime_wave20` | `2 passed` |
| Events | `test_event_runtime_wave21` | `2 passed` |
| Potions | `test_potion_runtime_wave8` | `6 passed, 1 ignored` |
| Potions | `test_potion_runtime_action_path` | `15 passed` |
| Relics | `test_relic_runtime_wave17` | `2 passed` |
| Relics | `test_dead_system_cleanup_wave22` | `1 passed` |
| Generated choice | `test_generated_choice_java_wave3` | `7 passed` |
| Orb timing | `test_orb_runtime_java_wave1` | `8 passed, 1 ignored` |
| Watcher edge cases | `test_card_runtime_watcher_wave26` | `3 passed` |

## 3. Merge-Gating Inconsistencies

These are the remaining blockers if we want the stronger claim `all gameplay content complete`.

### Finding G1
- Area: parity
- Severity: critical
- Confidence: high
- Scope: merge-gating
- Evidence: [shrines.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/events/shrines.rs:125), [test_event_runtime_wave19.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_event_runtime_wave19.rs:1), Java oracle `decompiled/java-src/com/megacrit/cardcrawl/events/shrines/GremlinMatchGame.java`
- Problem: `Match and Keep!` is still explicitly blocked. The Rust runtime does not yet model the Java card-grid reveal / match minigame.
- Recommended fix: add a dedicated event minigame runtime with hidden tile state, reveal/match resolution, and canonical event/reward integration.
- Test mapping: `test_event_runtime_wave19` should become behavioral minigame proof instead of blocker honesty.
- Worker slice: event minigame runtime

### Finding G2
- Area: parity
- Severity: medium
- Confidence: high
- Scope: merge-gating
- Evidence: [test_orb_runtime_java_wave1.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_orb_runtime_java_wave1.rs:254), Java oracle `decompiled/java-src/com/megacrit/cardcrawl/potions/LiquidMemories.java`
- Problem: `Liquid Memories` still returns deterministic top-discard cards instead of opening the full Java discard-choice selection.
- Recommended fix: add a discard-zone choice primitive that lets potions/cards select arbitrary discard cards through the canonical combat choice surface.
- Test mapping: retire the ignored `orb_wave1_liquid_memories_should_support_java_choice_selection` blocker once the discard-choice primitive lands.
- Worker slice: discard-choice primitive

### Finding G3
- Area: parity
- Severity: medium
- Confidence: high
- Scope: unsupported
- Evidence: [test_potion_runtime_wave8.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_potion_runtime_wave8.rs:218), Java oracle `decompiled/java-src/com/megacrit/cardcrawl/potions/SmokeBomb.java`
- Problem: `Smoke Bomb` legality is correct for bosses and normal combats, but the Java `BackAttack` / Surrounded positional caveat is still unmodeled because the Rust combat state has no positional representation.
- Recommended fix: either add positional combat state and close the caveat, or keep this limitation explicit in scope and docs.
- Test mapping: `wave8_smoke_bomb_back_attack_legality_remains_queued`
- Worker slice: positional combat state

## 4. Stale / Noisy Debt

### Finding S1
- Area: parity
- Severity: medium
- Confidence: high
- Scope: cleanup-only
- Evidence: current direct ignore count `75`, family table above, plus the cleanup-only card shells in [reflex.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/silent/reflex.rs:1), [tactician.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/silent/tactician.rs:1), [deusexmachina.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/watcher/deusexmachina.rs:1)
- Problem: the raw ignored backlog still overstates live parity debt. A meaningful portion is stale solved noise or cleanup-shell accounting rather than missing gameplay behavior.
- Recommended fix: run one follow-up ignored-test cleanup wave and re-bucket each ignored line into `live blocker`, `stale solved`, `cleanup-only`, or `post-merge enhancement`.
- Test mapping: source-wide `rg '#\\[ignore' packages/engine-rs/src/tests`
- Worker slice: ignored-test hygiene

### Finding S2
- Area: dead-system
- Severity: low
- Confidence: high
- Scope: cleanup-only
- Evidence: [effects/mod.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/effects/mod.rs:1), [effects/registry.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/effects/registry.rs:1), [test_dead_system_cleanup_wave22.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_dead_system_cleanup_wave22.rs:1)
- Problem: dead-system retirement is mostly done, but a few registry/export surfaces still make the architecture look older than it is.
- Recommended fix: trim remaining dead exports after the parity PR is scoped and stable.
- Test mapping: `test_dead_system_cleanup_wave22`
- Worker slice: dead-export cleanup

### Finding S3
- Area: architecture
- Severity: low
- Confidence: high
- Scope: cleanup-only
- Evidence: [engine.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/engine.rs:1), [test_card_runtime_nonplay_triggers_wave1.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_card_runtime_nonplay_triggers_wave1.rs:1)
- Problem: the cleanup-shell trio still exists as raw empty `effect_data` files even though the runtime behavior is proven elsewhere.
- Recommended fix: either collapse them into explicit runtime-owned marker defs or leave them as documented cleanup-only shells until the final registry cleanup wave.
- Test mapping: `test_card_runtime_nonplay_triggers_wave1`
- Worker slice: cleanup-shell normalization

## 5. Unsupported Backlog

Current explicit unsupported / partially scoped items:

- `Match and Keep!` Java minigame runtime
- `Smoke Bomb` back-attack positional legality

Intentional RL-facing deviations that are documented rather than treated as parity bugs:

- Neow always exposes `4` choices
- `NoteForYourself` future-run storage is canonical inside the runtime process rather than external profile-save persistence

See [DESIGN_DECISIONS.md](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/DESIGN_DECISIONS.md:1).

## 6. Post-Merge Backlog

These items should not block the supported-scope merge if scope stays honest, but they are still worth doing soon:

- ignored-test cleanup and de-noising
- cleanup-shell normalization for `Reflex`, `Tactician`, and `Deus Ex Machina`
- broader Java edge sweeps for generated-choice / Watcher follow-up families
- `Match and Keep!` full minigame if we choose to leave draft only after total all-content fidelity
- positional combat state if we want full `Smoke Bomb` legality fidelity

## 7. Edge-Case Annex: `Scrawl+`

This pass added direct engine-path proof for `Scrawl+` hand-limit behavior and the `Deus Ex Machina+` draw interaction in [test_card_runtime_watcher_wave26.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_card_runtime_watcher_wave26.rs:1).

Assumptions:

- initial hand size includes `Scrawl+`
- hand limit is `10`
- `Deus Ex Machina+` matters only if it is drawn during `Scrawl+`

| Initial hand incl `Scrawl+` | Hand after playing `Scrawl+` | No `Deus Ex` drawn | If next draw is `Deus Ex Machina+` |
| --- | --- | --- | --- |
| `3` | `2` | draw `8`, finish at `10` | `Deus` exhausts, add `3` Miracles, then draw `5` more, finish at `10` |
| `4` | `3` | draw `7`, finish at `10` | `Deus` exhausts, add `3` Miracles, then draw `4` more, finish at `10` |
| `5` | `4` | draw `6`, finish at `10` | `Deus` exhausts, add `3` Miracles, then draw `3` more, finish at `10` |
| `6` | `5` | draw `5`, finish at `10` | `Deus` exhausts, add `3` Miracles, then draw `2` more, finish at `10` |
| `7` | `6` | draw `4`, finish at `10` | `Deus` exhausts, add `3` Miracles, then draw `1` more, finish at `10` |
| `8` | `7` | draw `3`, finish at `10` | `Deus` exhausts, add `3` Miracles, finish at `10` immediately |

Late-draw cap proof also exists:

- if `Deus Ex Machina+` is drawn when current hand occupancy is `8`, only `2` Miracles fit
- if it is drawn when occupancy is `9`, only `1` Miracle fits

## 8. Training Appendix

The engine is strong enough to support a naive overnight baseline now. The current recommendation is still:

```bash
./scripts/training.sh start --games 4000 --workers 4 --batch 256 --asc 0 --headless --watchdog --sweep-config baseline_control
```

Why this is good enough for tonight:

- training smoke runs completed without engine/runtime crashes
- `training.sh status` and the active-run symlink behavior were hardened already
- the runtime/search surface now has green proof across `test_rl_contract` and `test_search_harness`

What still needs work before the training stack is “good,” not just “usable”:

- stronger run manifests and git/config provenance
- richer per-decision search diagnostics
- better stale-worker / stale-status visibility
- a cleaner training architecture that keeps only the good ideas from the current stack

Search-first restricted baseline idea:

- a Watcher A0 restricted-evaluation harness with upgrades/removes and no voluntary card adds is still a good diagnostic experiment
- it should be treated as an evaluation harness first, not tonight’s main baseline
