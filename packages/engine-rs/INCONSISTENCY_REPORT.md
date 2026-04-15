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
- `Liquid Memories` discard-choice selection is landed
- `Emotion Chip` timing is landed
- `Match and Keep!` minigame runtime is landed
- `Scrap Ooze` retry / flee / escalating relic-chance loop is landed
- `NoteForYourself` now runs as a real cross-run card-stash flow inside the canonical event / reward runtime

What is still open:

- the typed Defect multi-hit path still diverges from Java for `Barrage`, `Rip and Tear`, and `Thunder Strike`
- `Reinforced Body` still lacks Java repeated-block/X-cost parity
- `Smoke Bomb` still has one explicit positional caveat for Java `BackAttack` legality because the Rust combat model does not represent Surrounded/position state

Bottom line:

- If the claim is `supported runtime parity complete`, this branch is ready after cleanup/doc sync.
- If the claim is `all gameplay content complete`, that stronger claim is still false until section 3 is closed.
- Zero-skip answer: `no` — there are still `74` explicit `#[ignore]` tests in `packages/engine-rs/src/tests`.
- Java-clean answer: `no` — the `3` blockers above are still open on the current audited tree.

## 2. Quantified Baseline

### Inventory

| Metric | Current value | Notes |
| --- | ---: | --- |
| Registered card ids | `718` | Current registered-card read; raw source currently has `719` unique `id:` declarations including the non-registered `Unknown` fallback in `cards/mod.rs` |
| Typed event names | `52` | `62` `event(...)` call sites including continuation sub-states |
| Potion ids | `42` | Current source scan |
| Relic ids | `102` | Current source scan |
| Raw public gameplay-gap files | `0` | After excluding cleanup-only shells |
| Cleanup-only empty shells | `3` | `Reflex`, `Tactician`, `Deus Ex Machina` |
| Raw public `complex_hook` files | `0` | Current source scan |
| Blocked supported event ops | `0` | Current source scan |
| Explicit blocked event branches in source | `0` | none |
| Direct `#[ignore]` count in `src/tests` | `74` | Current source scan |

### Current status table

| Bucket | Current state |
| --- | --- |
| Fully supported | public card gameplay behavior, supported event runtime, Neow 4-choice action surface, Emotion Chip timing, potion action path, reward/runtime ordering, RL/search surfaces |
| Cleanup-only shells | `Reflex`, `Tactician`, `Deus Ex Machina` |
| Explicit blocked / not yet closed | Defect multi-hit family, `Reinforced Body` |
| Explicit semantic caveats | `Smoke Bomb` back-attack positional legality |

### Rust-vs-Java delta table

| Subsystem | Rust today | Java expectation | Current read |
| --- | --- | --- | --- |
| Shrine minigames | `Match and Keep!` indexed reveal/match loop | Java GremlinMatchGame-style hidden-card flow | closed in current branch |
| Exordium event state | `Scrap Ooze` retry / flee / escalating damage + relic chance | Java retry / flee / escalating damage + relic chance | closed in current branch |
| Defect multi-hit | zero-count hits clamped, random target reused per card | zero-hit no-op for some cards, fresh target per hit where applicable | real blocker |
| Defect X-cost block | `Reinforced Body` typed as one block gain | repeated block resolution per energy spent | real blocker |
| Potion legality | boss legality landed, no positional model | also forbid use under Java `BackAttack` / Surrounded caveat | explicit caveat |
| RL opening policy | Neow always exposes `4` choices | vanilla Java gates options by prior run state | intentional deviation |

### Ignored-test family summary

| Family | Current direct ignored count |
| --- | ---: |
| Watcher | `21` |
| Colorless / choice | `27` |
| Defect / orb | `7` |
| Ironclad | `1` |
| Potions | `1` |
| Other | `17` |

Some raw counts are intentionally noisy unless classified:

- the `3` raw empty public-card files are not gameplay gaps
- the `74` ignored tests include a mix of live blockers, stale solved lines, and cleanup-only noise
- there are no longer any explicit blocked event branches in source

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
| Events | `test_event_runtime_wave19` | `6 passed` |
| Events | `test_event_runtime_wave20` | `3 passed` |
| Events | `test_event_runtime_wave21` | `2 passed` |
| Potions | `test_potion_runtime_wave8` | `7 passed, 1 ignored` |
| Potions | `test_potion_runtime_action_path` | `15 passed` |
| Relics | `test_relic_runtime_wave17` | `2 passed` |
| Relics | `test_dead_system_cleanup_wave22` | `1 passed` |
| Generated choice | `test_generated_choice_java_wave3` | `7 passed` |
| Orb timing | `test_orb_runtime_java_wave1` | `9 passed` |
| Watcher edge cases | `test_card_runtime_watcher_wave26` | `3 passed` |

## 3. Merge-Gating Inconsistencies

These are the remaining blockers if we want the stronger claim `all gameplay content complete`.

### Finding G1
- Area: parity
- Severity: medium
- Confidence: high
- Scope: merge-gating
- Evidence: [card_effects.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/card_effects.rs:120), [card_effects.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/card_effects.rs:211), [test_card_runtime_defect_wave12.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_card_runtime_defect_wave12.rs:151), Java oracles `/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/Barrage.java`, `/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/ThunderStrike.java`, and `/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/defect/NewRipAndTearAction.java`
- Problem: the typed Defect multi-hit path diverges from Java in two ways. It clamps declared extra-hit counts with `.max(1)`, so zero-orb `Barrage` and zero-Lightning `Thunder Strike` still hit once in Rust. It also rolls `Target::RandomEnemy` only once per card, so `Rip and Tear` and `Thunder Strike` reuse one random target for every hit instead of retargeting per hit the way Java does.
- Recommended fix: allow zero-hit resolution for amount sources whose Java behavior is naturally a no-op at zero, and reroll random-enemy selection per hit for the affected Defect multi-hit cards.
- Test mapping: unignore or replace the zero-orb `Barrage`, per-hit-random `Rip and Tear`, and zero-Lightning / per-hit-random `Thunder Strike` cases in `test_card_runtime_defect_wave12`.
- Worker slice: `damage-debuff-pipeline`

### Finding G2
- Area: parity
- Severity: medium
- Confidence: high
- Scope: merge-gating
- Evidence: [reinforced_body.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/defect/reinforced_body.rs:1), [test_card_runtime_defect_wave9.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_card_runtime_defect_wave9.rs:47), Java oracle `/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/cards/blue/ReinforcedBody.java`
- Problem: `Reinforced Body` is still typed as a single `GainBlock(A::Block)` effect, but Java resolves block repeatedly through `ReinforcedBodyAction` based on energy spent. The current Rust surface does not yet model the repeated-block/X-cost semantics.
- Recommended fix: add a typed repeated-block/X-cost primitive and replace the blocker test with engine-path coverage for base and upgraded `Reinforced Body`.
- Test mapping: unignore `defect_wave9_reinforced_body_needs_typed_repeated_block_xcost_primitive` in `test_card_runtime_defect_wave9`.
- Worker slice: `damage-debuff-pipeline`

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
- Evidence: current direct ignore count `74`, family table above, plus the cleanup-only card shells in [reflex.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/silent/reflex.rs:1), [tactician.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/silent/tactician.rs:1), [deusexmachina.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/watcher/deusexmachina.rs:1)
- Problem: the raw ignored backlog still overstates live parity debt. A meaningful portion is stale solved noise or cleanup-shell accounting rather than missing gameplay behavior, including watcher placeholders for `Collect`, `Conjure Blade`, `Fasting`, `Judgement`, `Pressure Points`, `Wallop`, `Brilliance`, `Halt`, `Perseverance`, `Sands of Time`, and `Windmill Strike` that are already covered by later engine-path suites.
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

- `Smoke Bomb` back-attack positional legality
- Defect multi-hit parity for `Barrage`, `Rip and Tear`, and `Thunder Strike`
- `Reinforced Body` repeated-block/X-cost parity

Intentional RL-facing deviations that are documented rather than treated as parity bugs:

- Neow always exposes `4` choices
- `NoteForYourself` future-run storage is canonical inside the runtime process rather than external profile-save persistence

See [DESIGN_DECISIONS.md](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/DESIGN_DECISIONS.md:1).

## 6. Post-Merge Backlog

These items should not block the supported-scope merge if scope stays honest, but they are still worth doing soon:

- ignored-test cleanup and de-noising
- watcher stale-ignore cleanup for already landed `Collect`, `Conjure Blade`, `Fasting`, `Judgement`, `Pressure Points`, `Wallop`, `Brilliance`, `Halt`, `Perseverance`, `Sands of Time`, and `Windmill Strike`
- cleanup-shell normalization for `Reflex`, `Tactician`, and `Deus Ex Machina`
- broader Java edge sweeps for generated-choice / Watcher follow-up families
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
