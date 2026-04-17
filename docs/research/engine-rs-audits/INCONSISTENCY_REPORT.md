# Comprehensive Parity Inconsistency Report

Last updated: 2026-04-15  
Branch: `codex/universal-gameplay-runtime`

This is the canonical parity audit for `packages/engine-rs`. It reflects the live source tree after the staged runtime-trigger refactor, the stale-test cleanup wave, the final engine-PR cleanup pass, and the latest broad zero-skip freeze rerun.

## 1. Executive Summary

Current read:

- supported-scope runtime parity: `100%` on the audited matrix with documented intentional deviations
- all-content gameplay parity: `100%` on the audited matrix with documented intentional deviations
- supported-scope merge blockers: `0`
- all-content merge blockers: `0`

What is truly done:

- public gameplay-gap card tail: `0`
- raw public `complex_hook` tail: `0`
- blocked supported event-op tail: `0`
- final broad freeze result: `2189 passed`, `0 failed`, `0 ignored`
- registry-backed secondary card behavior now runs through typed runtime-trigger metadata instead of raw registry/tag ownership
- production/runtime/export code now has `0` raw `card.effects` reads and `0` live registry-dispatch symbols
- gameplay export now carries structured `runtime_traits`, `runtime_triggers`, and `play_hints` rather than semantic effect tags
- `Establishment` retained-cost parity is fixed
- `Match and Keep!`, `Scrap Ooze`, `NoteForYourself`, `Emotion Chip`, `Liquid Memories`, `Smoke Bomb`, Defect multi-hit parity, `Reinforced Body`, `Mutagenic Strength`, `DiscoveryAction`, `Chrysalis`, `Metamorphosis`, `Collect` timing, free-play X-cost handling, and persistent shop purge pricing are all landed on the canonical runtime path
- Neow action layer is real and intentionally always exposes `4` choices
- the stale solved ignore pile collapsed from `69` to `0`

What is still open:

- broader training-branch architecture planning now that the engine/export surface is typed and zero-skip green
- optional source-authoring cleanup outside the supported runtime surface

Bottom line:

- If the claim is `supported runtime parity complete`, this branch is ready to serve as the training-rebuild base after reviewer sign-off.
- If the claim is `all gameplay content complete`, the final broad freeze is green on the integrated branch and there is no currently confirmed unintended blocker left on the audited matrix.
- Zero-skip answer: `yes` — there are `0` explicit ignored tests.
- Java-clean answer: no currently confirmed unintended discrepancy remains on the targeted blocker matrix or the latest broad freeze rerun; intentional RL-facing deviations are documented in `DESIGN_DECISIONS.md`.

## 2. Quantified Baseline

### Inventory

| Metric | Current value | Notes |
| --- | ---: | --- |
| Registered card ids | `718` | existing registry audit baseline; card inventory did not change in this wave |
| Typed event names | `52` | existing event inventory baseline; event catalog did not change in this wave |
| Potion ids | `42` | source scan |
| Relic ids | `102` | source scan |
| Raw public gameplay-gap files | `0` | after excluding runtime-trigger-only non-play cards |
| Runtime-trigger-only cards with empty primary play body | `3` | `Reflex`, `Tactician`, `Deus Ex Machina` |
| Raw public `complex_hook` files | `0` | current source scan |
| Blocked supported event ops | `0` | current source scan |
| Explicit blocked event branches in source | `0` | current source scan |
| Direct ignored tests in `src/tests` | `0` | current source scan |
| Production raw `card.effects` reads | `0` | current source scan |
| Live registry-dispatch symbols | `0` | current source scan |
| Typed runtime-trigger cutover | `landed` | migrated secondary behavior now reads from `CardRuntimeTraits` / `CardRuntimeTrigger` and not raw tag checks for the migrated families |
| Final broad freeze | `2189 / 2189` | latest integrated local run |

### Current status table

| Bucket | Current state |
| --- | --- |
| Fully supported | public gameplay-gap cards, supported event runtime, Neow action surface, potion action path, reward/runtime ordering, RL/search surfaces |
| Runtime-trigger-only cards | `Reflex`, `Tactician`, `Deus Ex Machina` |
| Explicit gameplay blockers | none currently confirmed on the targeted blocker matrix or the broad freeze rerun |
| Cleanup-only ignores | none |

### Rust-vs-Java delta table

| Subsystem | Rust today | Java expectation | Current read |
| --- | --- | --- | --- |
| Shrine minigames | `Match and Keep!` indexed reveal/match loop | Java GremlinMatchGame-style hidden-card flow | closed |
| Exordium event state | `Scrap Ooze` retry / flee / escalating damage + relic chance | Java retry / flee / escalating damage + relic chance | closed |
| Defect multi-hit | zero-hit and per-hit-target behavior covered | zero-hit no-op where appropriate, fresh target semantics where applicable | closed |
| Potion legality | boss and `BackAttack` legality covered | forbid use under Java legality gates | closed |
| Retain-cost powers | `Establishment` modifies retained-card combat cost across turns | Java `EstablishmentPower` does the same | closed |
| Secondary card-runtime ownership | typed runtime-trigger metadata plus derived compat tags | card-owned secondary behavior without registry/tag ambiguity | closed for the migrated families |
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

- the `3` raw empty public-card files are intentional runtime-trigger-only non-play cards, not gameplay gaps; their runtime semantics live in typed runtime-trigger metadata and are proven directly in `test_card_runtime_nonplay_triggers_wave1`
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
| Runtime-trigger cutover | `test_runtime_inline_cutover_wave5` | green |
| Runtime-trigger cutover | `test_card_runtime_nonplay_triggers_wave1` | green |
| Runtime-trigger cutover | `test_card_runtime_support_wave1` | green |
| Broad class parity | `test_cards_ironclad` | green |
| Broad class parity | `test_cards_defect` | green |
| Broad class parity | `test_cards_silent` | green |
| Broad class parity | `test_cards_watcher` | green |
| Watcher integration | `test_card_runtime_watcher_wave24` | green |
| X-count integration | `test_card_runtime_xcount_wave1` | green |
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

There are no currently confirmed merge-gating findings on the integrated zero-skip tree after the final `2189 / 2189` broad freeze.

The last known blocker sweep is now closed by passing engine-path proof:

- `Collect` pre-draw Miracle timing in [test_card_runtime_watcher_wave24.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_card_runtime_watcher_wave24.rs:154)
- free-play X-cost energy preservation in [test_card_runtime_xcount_wave1.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_card_runtime_xcount_wave1.rs:105)
- `Emotion Chip` multi-orb and `Cables` fidelity in [test_orb_runtime_java_wave1.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_orb_runtime_java_wave1.rs:239)
- persistent shop purge pricing in [test_run_parity.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_run_parity.rs:91)

## 4. Stale / Noisy Debt

### Finding S1
- Area: parity
- Severity: low
- Confidence: high
- Scope: cleanup-only
- Evidence: current direct ignore count `0`; prior stale-ignore baseline `69`
- Problem: the branch no longer has any ignored-test backlog, but older docs and review context still describe that larger stale world.
- Recommended fix: keep the canonical docs and PR body synced to the new zero-ignore baseline and stop referencing the old `69` count.
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
- Evidence: [reflex.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/silent/reflex.rs:1), [tactician.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/silent/tactician.rs:1), [deusexmachina.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/watcher/deusexmachina.rs:1), [test_card_runtime_nonplay_triggers_wave1.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_card_runtime_nonplay_triggers_wave1.rs:1)
- Problem: three non-play cards still have empty primary play bodies in source, which can look suspicious if the typed runtime-trigger ownership is not documented alongside them.
- Recommended fix: keep them documented and tested as intentional runtime-trigger-only card defs; no gameplay migration work remains for this trio.
- Test mapping: `test_card_runtime_nonplay_triggers_wave1`
- Worker slice: runtime-only authoring clarity

## 5. Intentional Deviations

Intentional RL-facing deviations that are documented rather than treated as parity bugs:

- Neow always exposes `4` choices
- `NoteForYourself` future-run storage is canonical inside the runtime process rather than external profile-save persistence

See [DESIGN_DECISIONS.md](../../../packages/engine-rs/DESIGN_DECISIONS.md).

## 6. Post-Merge Backlog

These items should not block a supported-scope merge if scope stays honest, but they are still worth doing soon:

- relic bridge retirement in dead-system cleanup waves `18` and `19`
- broader generated-choice and generated-card fidelity sweeps as confidence work rather than blocker work
- training-branch architecture planning now that the branch is zero-skip and broad-freeze green

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
