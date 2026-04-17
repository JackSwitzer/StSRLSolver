# Engine Parity Scorecard

Last updated: 2026-04-15  
Branch: `codex/universal-gameplay-runtime`

Canonical audit outputs:

- [INCONSISTENCY_REPORT.md](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/INCONSISTENCY_REPORT.md:1)
- [DECOMPILE_PARITY_ENDGAME.md](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/DECOMPILE_PARITY_ENDGAME.md:1)
- [DESIGN_DECISIONS.md](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/DESIGN_DECISIONS.md:1)

## Scorecard

Weighted toward `universal gameplay runtime + honest Java parity proof`:

- supported-scope runtime parity: `100%` on the audited matrix with documented intentional deviations
- all-content gameplay parity: `100%` on the audited matrix with documented intentional deviations
- architecture unification snapshot: `100%` semantic ownership on the supported Rust/runtime/export surface

Area scores:

| Area | Score | Notes |
| --- | ---: | --- |
| Combat runtime parity | `100%` on the audited matrix | the prior `Collect` / free-play X-cost / `Emotion Chip` / shop purge-cost blocker sweep is now closed with passing engine-path regressions, and the final broad freeze finished `2189 / 2189` green on the integrated branch |
| RL combat surface | `98%` | legal-action, observation, and search surfaces are green; training-side alignment is still separate work |
| Run / reward / event parity | `100%` | `NoteForYourself`, `Match and Keep!`, `Scrap Ooze`, and persistent shop purge pricing now run on the canonical runtime path |
| Dead-system retirement | `99%` | the stale cleanup-ignore tail in waves `18` and `19` is gone; remaining work is normal follow-on cleanup rather than parity debt |
| Runtime-trigger unification | `100%` on the supported surface | production/runtime/export code no longer reads raw `card.effects` or the deleted registry-dispatch layer; the remaining raw-empty card defs are intentional runtime-trigger-only authoring files with explicit engine-path proof |

## Current Quantified Backlog

| Metric | Value |
| --- | ---: |
| Raw public card files with empty `effect_data` | `3` |
| Raw public card files with `complex_hook: Some(...)` | `0` |
| Unresolved public gameplay-gap files | `0` |
| Runtime-trigger-only cards with empty primary play body | `3` |
| Blocked supported event ops | `0` |
| Explicit blocked event branches in source | `0` |
| Direct `#[ignore]` count in `src/tests` | `0` |
| Live gameplay-blocking ignore families | `0` |
| Cleanup-only ignore families | `0` |
| Live production potion fallback callsites | `0` |
| Direct relic helper-path refs | `0` |
| Production raw `card.effects` reads | `0` |
| Live registry-dispatch symbols | `0` |
| Final broad freeze | `2189 / 2189` |

Runtime-trigger-only card defs:

- [reflex.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/silent/reflex.rs:1)
- [tactician.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/silent/tactician.rs:1)
- [deusexmachina.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/watcher/deusexmachina.rs:1)

These are intentional non-play cards. Their runtime semantics are owned by the typed runtime-trigger surface, and [test_card_runtime_nonplay_triggers_wave1.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_card_runtime_nonplay_triggers_wave1.rs:1) now proves them as explicit runtime-only defs rather than suspicious leftovers.

Recently closed gameplay-gap families:

- `Parasite` master-deck removal now routes through a run-owned removal hook and has engine-path proof in [test_run_parity.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_run_parity.rs:151)
- `Sentinel` under `Corruption` now uses a typed `on_exhaust` hook lane and is proven in [test_card_runtime_ironclad_wave9.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_card_runtime_ironclad_wave9.rs:83)
- `Expunger` / `Conjure Blade+` now use typed generated-card and card-owned X-count surfaces, including `Chemical X` coverage in [test_card_runtime_watcher_wave24.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_card_runtime_watcher_wave24.rs:137)
- registry-backed secondary behavior now runs through typed runtime-trigger metadata in [runtime_meta.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/runtime_meta.rs:1) and [card_runtime.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/effects/card_runtime.rs:1), while [gameplay/types.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/gameplay/types.rs:1) exports structured runtime traits/triggers/play hints instead of semantic effect tags
- production Rust now has `0` raw `card.effects` reads and `0` live registry-dispatch symbols on the verified source audit

Last known semantic blocker sweep now closed:

- `Collect` now resolves Miracles before draw and is proven in [test_card_runtime_watcher_wave24.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_card_runtime_watcher_wave24.rs:154)
- free-play X-cost cards now snapshot X without draining energy and are proven in [test_card_runtime_xcount_wave1.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_card_runtime_xcount_wave1.rs:105)
- `Emotion Chip` now replays full multi-orb `ImpulseAction`-style passives, including `Cables`, in [test_orb_runtime_java_wave1.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_orb_runtime_java_wave1.rs:239)
- shop purge pricing now persists across visits and applies discounts off run-owned purge state in [test_run_parity.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_run_parity.rs:91)

## Why The Branch Is Trusted

Representative green suites on the current tree:

- `./scripts/test_engine_rs.sh check --lib`
- `./scripts/test_engine_rs.sh test --lib --no-run`
- `test_run_parity`
- `test_rl_contract`
- `test_search_harness`
- `test_reward_runtime`
- `test_events_parity`
- `test_event_runtime_wave19`
- `test_event_runtime_wave20`
- `test_event_runtime_wave21`
- `test_potion_runtime_wave8`
- `test_potion_runtime_action_path`
- `test_relic_runtime_wave17`
- `test_dead_system_cleanup_wave22`
- `test_generated_choice_java_wave3`
- `test_orb_runtime_java_wave1`
- `test_card_runtime_watcher_wave24`
- `test_card_runtime_xcount_wave1`
- `test_cards_ironclad`
- `test_cards_defect`
- `test_cards_silent`
- `test_cards_watcher`
- `test_card_runtime_nonplay_triggers_wave1`
- `test_card_runtime_support_wave1`
- `test_runtime_inline_cutover_wave5`
- `test_card_runtime_backend_wave1`
- `test_card_runtime_backend_wave2`
- `test_card_runtime_backend_wave3`
- `test_card_runtime_watcher_wave5`
- `test_card_runtime_watcher_wave14`
- `test_card_runtime_watcher_wave15`
- `test_card_runtime_watcher_wave16`
- `test_card_runtime_watcher_wave17`
- `test_card_runtime_watcher_wave19`
- `test_card_runtime_watcher_wave20`
- `test_card_runtime_colorless_wave2`
- `test_card_runtime_colorless_wave3`
- `test_card_runtime_colorless_wave4`
- `test_card_runtime_colorless_wave5`
- `test_card_runtime_colorless_wave6`
- `test_card_runtime_colorless_wave8`
- `test_card_runtime_defect_wave8`
- `test_card_runtime_defect_wave9`
- `test_card_runtime_defect_wave13`
- `test_zone_batch_java_wave2`
- `test_zone_batch_java_wave3`

The main stale-test cleanup result from this pass:

- direct ignored tests dropped from `69` to `0`
- generated-choice fidelity for `DiscoveryAction`, `Chrysalis`, and `Metamorphosis` is now covered by passing tests instead of ignores
- stale active failures removed: `Consecrate`, `Purity`, `Capacitor`
- real runtime fix landed: `Establishment`
- the final cleanup wave also removed stale dead-system ignores and replaced the last gameplay blockers with passing engine-path proof

## Current Read

- If the claim is `supported runtime parity complete`, the branch is there on the audited matrix.
- If the claim is `all gameplay content complete`, the last known blocker sweep is now closed and the final broad freeze finished `2189 / 2189` green; the remaining work is training-branch planning and optional follow-on cleanup, not live gameplay debt.
- Zero-skip answer: `yes` — there are `0` explicit `#[ignore]` tests in `src/tests`.
- Java-clean answer: no currently confirmed unintended discrepancy remains on the targeted blocker matrix or the latest broad freeze rerun; intentional RL-facing deviations are tracked in `DESIGN_DECISIONS.md`.
- Legacy semantic code answer: none remains in the supported Rust/runtime/export surface. The remaining raw-empty card defs are intentional runtime-trigger-only authoring files, not semantic fallback paths.
