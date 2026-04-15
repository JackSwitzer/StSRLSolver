# Engine Parity Scorecard

Last updated: 2026-04-15  
Branch: `codex/universal-gameplay-runtime`

Canonical audit outputs:

- [INCONSISTENCY_REPORT.md](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/INCONSISTENCY_REPORT.md:1)
- [DECOMPILE_PARITY_ENDGAME.md](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/DECOMPILE_PARITY_ENDGAME.md:1)
- [DESIGN_DECISIONS.md](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/DESIGN_DECISIONS.md:1)

## Scorecard

Weighted toward `universal gameplay runtime + honest Java parity proof`:

- supported-scope runtime parity: `99%`
- all-content gameplay parity: `99%`
- architecture unification snapshot: `99%`

Area scores:

| Area | Score | Notes |
| --- | ---: | --- |
| Combat runtime parity | `99%` | `Establishment` is fixed and the stale watcher/colorless skip pile is largely gone; the remaining gaps are narrow Java-cited families |
| RL combat surface | `98%` | legal-action, observation, and search surfaces are green; training-side alignment is still separate work |
| Run / reward / event parity | `99%` | `NoteForYourself`, `Match and Keep!`, and `Scrap Ooze` run on the canonical event/runtime path |
| Dead-system retirement | `98%` | gameplay helper debt is mostly retired, but two relic-bridge cleanup ignores remain |

## Current Quantified Backlog

| Metric | Value |
| --- | ---: |
| Raw public card files with empty `effect_data` | `3` |
| Raw public card files with `complex_hook: Some(...)` | `0` |
| Unresolved public gameplay-gap files | `0` |
| Cleanup-only card shells | `3` |
| Blocked supported event ops | `0` |
| Explicit blocked event branches in source | `0` |
| Direct `#[ignore]` count in `src/tests` | `9` |
| Live gameplay-blocking ignore families | `7` |
| Cleanup-only ignore families | `2` |
| Live production potion fallback callsites | `0` |
| Direct relic helper-path refs | `0` |

Cleanup-only card shells:

- [reflex.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/silent/reflex.rs:1)
- [tactician.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/silent/tactician.rs:1)
- [deusexmachina.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/watcher/deusexmachina.rs:1)

Remaining real gameplay blocker families:

- `Parasite` master-deck removal max-HP semantics
- `DiscoveryAction` potency-sensitive generation count
- `Chrysalis` random upgraded Skill generation fidelity
- `Metamorphosis` random upgraded Attack generation fidelity
- `Sentinel` under `Corruption` exhaust-trigger energy refund parity
- `Expunger` temp-card X-count / repeated-hit state fidelity
- `Mutagenic Strength` combat-start temporary Strength timing

Cleanup-only remaining ignores:

- relic bridge retirement in [test_dead_system_cleanup_wave18.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_dead_system_cleanup_wave18.rs:52)
- relic bridge retirement in [test_dead_system_cleanup_wave19.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_dead_system_cleanup_wave19.rs:70)

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

- direct ignored tests dropped from `69` to `9`
- stale active failures removed: `Consecrate`, `Purity`, `Capacitor`
- real runtime fix landed: `Establishment`

## Current Read

- If the claim is `supported runtime parity complete`, the branch is effectively there.
- If the claim is `all gameplay content complete`, the branch still needs the `7` remaining gameplay families above.
- Zero-skip answer: `no` — there are still `9` explicit `#[ignore]` tests in `src/tests`.
- Java-clean answer: `no` — the branch still has a small, explicit Java-cited tail rather than a broad unknown.
