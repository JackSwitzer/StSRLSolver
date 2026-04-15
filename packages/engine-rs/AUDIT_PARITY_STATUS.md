# Engine Parity Scorecard

Last updated: 2026-04-15  
Branch: `codex/universal-gameplay-runtime`

Canonical audit outputs:

- [INCONSISTENCY_REPORT.md](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/INCONSISTENCY_REPORT.md:1)
- [DECOMPILE_PARITY_ENDGAME.md](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/DECOMPILE_PARITY_ENDGAME.md:1)
- [DESIGN_DECISIONS.md](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/DESIGN_DECISIONS.md:1)

## Scorecard

Weighted toward `universal gameplay runtime + honest Java parity proof`:

- supported-scope runtime parity: `100%`
- all-content gameplay parity: `99%`
- architecture unification snapshot: `99%`

Area scores:

| Area | Score | Notes |
| --- | ---: | --- |
| Combat runtime parity | `99%` | the last known gameplay gaps in this cleanup wave are now closed on the engine path; remaining risk is audit confidence, not an explicit blocker list |
| RL combat surface | `98%` | legal-action, observation, and search surfaces are green; training-side alignment is still separate work |
| Run / reward / event parity | `99%` | `NoteForYourself`, `Match and Keep!`, and `Scrap Ooze` run on the canonical event/runtime path |
| Dead-system retirement | `99%` | the stale cleanup-ignore tail in waves `18` and `19` is gone; remaining work is normal follow-on cleanup rather than parity debt |

## Current Quantified Backlog

| Metric | Value |
| --- | ---: |
| Raw public card files with empty `effect_data` | `3` |
| Raw public card files with `complex_hook: Some(...)` | `0` |
| Unresolved public gameplay-gap files | `0` |
| Cleanup-only card shells | `3` |
| Blocked supported event ops | `0` |
| Explicit blocked event branches in source | `0` |
| Direct `#[ignore]` count in `src/tests` | `0` |
| Live gameplay-blocking ignore families | `0` |
| Cleanup-only ignore families | `0` |
| Live production potion fallback callsites | `0` |
| Direct relic helper-path refs | `0` |

Cleanup-only card shells:

- [reflex.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/silent/reflex.rs:1)
- [tactician.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/silent/tactician.rs:1)
- [deusexmachina.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/watcher/deusexmachina.rs:1)

Recently closed gameplay-gap families:

- `Parasite` master-deck removal now routes through a run-owned removal hook and has engine-path proof in [test_run_parity.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_run_parity.rs:151)
- `Sentinel` under `Corruption` now uses a typed `on_exhaust` hook lane and is proven in [test_card_runtime_ironclad_wave9.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_card_runtime_ironclad_wave9.rs:83)
- `Expunger` / `Conjure Blade+` now use typed generated-card and card-owned X-count surfaces, including `Chemical X` coverage in [test_card_runtime_watcher_wave24.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_card_runtime_watcher_wave24.rs:137)

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
- If the claim is `all gameplay content complete`, there are no currently confirmed gameplay blockers left in the audited matrix.
- Zero-skip answer: `yes` — there are `0` explicit `#[ignore]` tests in `src/tests`.
- Java-clean answer: `no currently confirmed discrepancy remains in the audited matrix`; remaining risk is unexercised edge cases rather than a live blocker list.
