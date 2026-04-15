# Comprehensive Parity Inconsistency Report

Last updated: 2026-04-15  
Branch: `codex/universal-gameplay-runtime`

This is the canonical parity audit for `packages/engine-rs`. It reflects the live source tree after the stale-test cleanup wave, not the older `69`-ignore / `Establishment`-blocked snapshot.

## 1. Executive Summary

Current read:

- supported-scope runtime parity: `~100%`
- all-content gameplay parity: `~98%`
- supported-scope merge blockers: `0`
- all-content merge blockers: `4` semantic families

What is truly done:

- public gameplay-gap card tail: `0`
- raw public `complex_hook` tail: `0`
- blocked supported event-op tail: `0`
- `Establishment` retained-cost parity is fixed
- `Match and Keep!`, `Scrap Ooze`, `NoteForYourself`, `Emotion Chip`, `Liquid Memories`, `Smoke Bomb`, Defect multi-hit parity, `Reinforced Body`, `Mutagenic Strength`, `DiscoveryAction`, `Chrysalis`, and `Metamorphosis` are all landed on the canonical runtime path
- Neow action layer is real and intentionally always exposes `4` choices
- the stale solved ignore pile collapsed from `69` to `5`

What is still open:

- `Collect` timing parity
- free-play X-cost energy preservation
- `Emotion Chip` full `ImpulseAction` fidelity
- shop purge-cost persistence
- then one final broad parity freeze and doc/PR synchronization

Bottom line:

- If the claim is `supported runtime parity complete`, this branch is ready after final doc/PR sync.
- If the claim is `all gameplay content complete`, that stronger claim is still false until the `4` semantic blockers above are closed.
- Zero-skip answer: `yes` — there are `0` explicit ignored tests.
- Java-clean answer: `no` — the final broad audit confirmed a short explicit blocker list.

## 2. Quantified Baseline

### Inventory

| Metric | Current value | Notes |
| --- | ---: | --- |
| Registered card ids | `718` | existing registry audit baseline; card inventory did not change in this wave |
| Typed event names | `52` | existing event inventory baseline; event catalog did not change in this wave |
| Potion ids | `42` | source scan |
| Relic ids | `102` | source scan |
| Raw public gameplay-gap files | `0` | after excluding cleanup-only shells |
| Cleanup-only empty shells | `3` | `Reflex`, `Tactician`, `Deus Ex Machina` |
| Raw public `complex_hook` files | `0` | current source scan |
| Blocked supported event ops | `0` | current source scan |
| Explicit blocked event branches in source | `0` | current source scan |
| Direct ignored tests in `src/tests` | `0` | current source scan |

### Current status table

| Bucket | Current state |
| --- | --- |
| Fully supported | public gameplay-gap cards, supported event runtime, Neow action surface, potion action path, reward/runtime ordering, RL/search surfaces |
| Cleanup-only shells | `Reflex`, `Tactician`, `Deus Ex Machina` |
| Explicit gameplay blockers | `Collect`, free-play X-cost, `Emotion Chip`, shop purge-cost persistence |
| Cleanup-only ignores | none |

### Rust-vs-Java delta table

| Subsystem | Rust today | Java expectation | Current read |
| --- | --- | --- | --- |
| Shrine minigames | `Match and Keep!` indexed reveal/match loop | Java GremlinMatchGame-style hidden-card flow | closed |
| Exordium event state | `Scrap Ooze` retry / flee / escalating damage + relic chance | Java retry / flee / escalating damage + relic chance | closed |
| Defect multi-hit | zero-hit and per-hit-target behavior covered | zero-hit no-op where appropriate, fresh target semantics where applicable | closed |
| Potion legality | boss and `BackAttack` legality covered | forbid use under Java legality gates | closed |
| Retain-cost powers | `Establishment` modifies retained-card combat cost across turns | Java `EstablishmentPower` does the same | closed |
| RL opening policy | Neow always exposes `4` choices | vanilla Java gates options by prior run state | intentional deviation |

### Ignored-test family summary

| Family | Current direct ignored count |
| --- | ---: |
| Generated choice / card generation | `0` |
| Card runtime parity | `0` |
| Dead-system cleanup | `0` |
| Watcher stale solved noise | `0` |
| Colorless stale solved noise | `0` |
| Defect stale solved noise | `0` |

Some raw counts are intentionally noisy unless classified:

- the `3` raw empty public-card files are cleanup-only shells, not gameplay gaps
- the ignore backlog is fully collapsed on the live source tree

### Why we believe the engine works

Representative green suites on the current local tree:

| Area | Suite | Result |
| --- | --- | --- |
| Wrapper gate | `./scripts/test_engine_rs.sh check --lib` | green |
| Wrapper gate | `./scripts/test_engine_rs.sh test --lib --no-run` | green |
| Run / RL | `test_run_parity` | green |
| Run / RL | `test_rl_contract` | green |
| Search | `test_search_harness` | green |
| Rewards | `test_reward_runtime` | green |
| Events | `test_events_parity` | green |
| Events | `test_event_runtime_wave19` | green |
| Events | `test_event_runtime_wave20` | green |
| Events | `test_event_runtime_wave21` | green |
| Potions | `test_potion_runtime_wave8` | green |
| Potions | `test_potion_runtime_action_path` | green |
| Relics | `test_relic_runtime_wave17` | green |
| Dead-system | `test_dead_system_cleanup_wave22` | green |
| Generated choice | `test_generated_choice_java_wave3` | green |
| Orb timing | `test_orb_runtime_java_wave1` | green |
| Watcher retain / registry cleanup | `test_card_runtime_watcher_wave5` | green |
| Watcher stale-ignore cleanup | `test_card_runtime_watcher_wave14` | green |
| Watcher stale-ignore cleanup | `test_card_runtime_watcher_wave15` | green |
| Watcher stale-ignore cleanup | `test_card_runtime_watcher_wave16` | green |
| Watcher stale-ignore cleanup | `test_card_runtime_watcher_wave17` | green |
| Watcher stale-ignore cleanup | `test_card_runtime_watcher_wave19` | green |
| Watcher stale-ignore cleanup | `test_card_runtime_watcher_wave20` | green |
| Colorless cleanup | `test_card_runtime_colorless_wave2` | green |
| Colorless cleanup | `test_card_runtime_colorless_wave3` | green |
| Colorless cleanup | `test_card_runtime_colorless_wave4` | green |
| Colorless cleanup | `test_card_runtime_colorless_wave5` | green |
| Colorless cleanup | `test_card_runtime_colorless_wave6` | green |
| Colorless cleanup | `test_card_runtime_colorless_wave8` | green |
| Defect cleanup | `test_card_runtime_defect_wave8` | green |
| Defect cleanup | `test_card_runtime_defect_wave9` | green |
| Defect cleanup | `test_card_runtime_defect_wave13` | green |
| Legality cleanup | `test_zone_batch_java_wave2` | green |
| Registry / batch cleanup | `test_zone_batch_java_wave3` | green |

## 3. Confirmed Merge-Gating Findings

### Finding G1
- Area: parity
- Severity: medium
- Confidence: high
- Scope: merge-gating
- Evidence: [engine.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/engine.rs:1145), [CollectPower.java](/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/powers/CollectPower.java:50), [PlayerTurnEffect.java](/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/vfx/PlayerTurnEffect.java:50)
- Problem: Rust grants `Collect` Miracles after the start-of-turn draw, while Java queues them from `onEnergyRecharge()` before draw.
- Recommended fix: move the `Collect` one-shot resolution into the pre-draw energy-recharge phase and add an engine-path timing test.
- Test mapping: queued test for `Collect` with near-hand-cap start-of-turn ordering
- Worker slice: watcher turn-start timing

### Finding G2
- Area: parity
- Severity: medium
- Confidence: high
- Scope: merge-gating
- Evidence: [card_effects.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/card_effects.rs:281), Java X-cost actions such as [TempestAction.java](/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/defect/TempestAction.java:31)
- Problem: free-to-play X-cost cards still drain all energy in Rust even though Java snapshots X without spending energy when the card is free.
- Recommended fix: preserve the current-energy snapshot for X while gating the actual energy drain on `!card_inst.is_free()`.
- Test mapping: queued free-play X-cost proof on `Whirlwind`, `Tempest`, or `Skewer`
- Worker slice: shared X-cost resolution

### Finding G3
- Area: parity
- Severity: medium
- Confidence: high
- Scope: merge-gating
- Evidence: [engine.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/engine.rs:1079), [emotion_chip.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/relics/defs/emotion_chip.rs:1), [ImpulseAction.java](/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/actions/defect/ImpulseAction.java:20)
- Problem: Rust models `Emotion Chip` as a single front-orb pulse, but Java runs `ImpulseAction` across all orbs and repeats the front orb when `Cables` is present.
- Recommended fix: replace the current one-slot replay with full `ImpulseAction`-style orb iteration and add multi-orb/Cables proof.
- Test mapping: queued multi-orb `Emotion Chip` test with and without `Cables`
- Worker slice: defect orb turn-start fidelity

### Finding G4
- Area: parity
- Severity: medium
- Confidence: high
- Scope: merge-gating
- Evidence: [run.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/run.rs:2516), [ShopScreen.java](/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/shop/ShopScreen.java:223), [ShopScreen.java](/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/shop/ShopScreen.java:275)
- Problem: Rust derives shop purge cost from `combats_won`, while Java uses persistent `purgeCost` / `actualPurgeCost` that increments only when a purge happens.
- Recommended fix: add run-owned persistent purge-cost state and update the shop tests to match the Java contract.
- Test mapping: queued run-parity proof for purge-cost persistence across shops
- Worker slice: run shop economy parity

## 4. Stale / Noisy Debt

### Finding S1
- Area: parity
- Severity: low
- Confidence: high
- Scope: cleanup-only
- Evidence: current direct ignore count `5`; prior stale-ignore baseline `69`
- Problem: the branch no longer has a huge stale-ignore backlog, but older docs and review context still describe that larger stale world.
- Recommended fix: keep the canonical docs and PR body synced to the new `5`-ignore baseline and stop referencing the old `69` count.
- Test mapping: source-wide ignore scan
- Worker slice: audit/doc reconciliation

### Finding S2
- Area: dead-system
- Severity: low
- Confidence: high
- Scope: cleanup-only
- Evidence: [test_dead_system_cleanup_wave18.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_dead_system_cleanup_wave18.rs:1), [test_dead_system_cleanup_wave19.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_dead_system_cleanup_wave19.rs:1)
- Problem: the old cleanup-only ignore tail is gone, but the docs and reviewer context still describe it as active debt.
- Recommended fix: keep the canonical docs and PR narrative synced to the current zero-ignore tree so stale cleanup debt is not mistaken for an engine blocker.
- Test mapping: `test_dead_system_cleanup_wave18`, `test_dead_system_cleanup_wave19`
- Worker slice: audit/doc reconciliation

### Finding S3
- Area: architecture
- Severity: low
- Confidence: high
- Scope: cleanup-only
- Evidence: [reflex.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/silent/reflex.rs:1), [tactician.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/silent/tactician.rs:1), [deusexmachina.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/watcher/deusexmachina.rs:1)
- Problem: the cleanup-shell trio still exists as raw empty `effect_data` files even though their runtime behavior is already proven elsewhere.
- Recommended fix: leave them documented as cleanup-only shells or collapse them into explicit runtime-owned marker defs in a later normalization pass.
- Test mapping: non-play trigger/runtime suites
- Worker slice: cleanup-shell normalization

## 5. Intentional Deviations

Intentional RL-facing deviations that are documented rather than treated as parity bugs:

- Neow always exposes `4` choices
- `NoteForYourself` future-run storage is canonical inside the runtime process rather than external profile-save persistence

See [DESIGN_DECISIONS.md](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/DESIGN_DECISIONS.md:1).

## 6. Post-Merge Backlog

These items should not block a supported-scope merge if scope stays honest, but they are still worth doing soon:

- relic bridge retirement in dead-system cleanup waves `18` and `19`
- cleanup-shell normalization for `Reflex`, `Tactician`, and `Deus Ex Machina`
- broader generated-choice and generated-card fidelity sweeps after the explicit blockers above close
- broad audit freezing now that the branch is completely ignore-free

## 7. Edge-Case Annex: `Scrawl+`

Direct engine-path proof for `Scrawl+` hand-limit behavior and the `Deus Ex Machina+` draw interaction exists in [test_card_runtime_watcher_wave26.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_card_runtime_watcher_wave26.rs:1).

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

The engine is strong enough to support a naive overnight baseline now, but the training stack is not yet “good,” only “usable.”

Current recommendation:

```bash
./scripts/training.sh start --games 4000 --workers 4 --batch 256 --asc 0 --headless --watchdog --sweep-config baseline_control
```

Why this is good enough for tonight:

- training smoke runs completed without engine/runtime crashes
- `training.sh status` and active-run status handling were already hardened
- the runtime/search surface now has green proof across `test_rl_contract` and `test_search_harness`

What still needs work before the training stack is “good”:

- align training with the live Neow surface instead of defaulting to `skip_neow=True`
- document and version the Rust-vs-Python observation contract
- add a training-side restriction overlay for curriculum rules like “no card rewards”
- add stronger run manifests and git/config provenance
- add richer per-decision search diagnostics
- improve stale-worker / stale-status visibility
