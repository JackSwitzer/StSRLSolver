# Engine Parity Scorecard

Last updated: 2026-04-14  
Branch: `codex/universal-gameplay-runtime`

Canonical audit outputs:

- [INCONSISTENCY_REPORT.md](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/INCONSISTENCY_REPORT.md:1)
- [DECOMPILE_PARITY_ENDGAME.md](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/DECOMPILE_PARITY_ENDGAME.md:1)
- [DESIGN_DECISIONS.md](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/DESIGN_DECISIONS.md:1)

## Scorecard

Weighted completion toward `universal gameplay runtime + decision-complete RL loop`:

- supported-scope runtime parity: `99%`
- all-content gameplay parity: `98%`
- architecture unification snapshot: `99%`

Area scores:

| Area | Score | Notes |
| --- | ---: | --- |
| Combat runtime parity | `99%` | Public gameplay-gap card tail is closed |
| RL combat surface | `98%` | `Neow`, reward screen, decision context, and search surfaces are green |
| Run / reward / event parity | `98%` | `Scrap Ooze` and `NoteForYourself` now land; `Match and Keep!` still blocked |
| Dead-system retirement | `98%` | Helper-path production debt is effectively gone |

## Current Quantified Backlog

| Metric | Value |
| --- | ---: |
| Raw public card files with empty `effect_data` | `3` |
| Raw public card files with `complex_hook: Some(...)` | `0` |
| Unresolved public gameplay-gap files | `0` |
| Cleanup-only card shells | `3` |
| Blocked supported event ops | `0` |
| Explicit blocked event branches in source | `1` |
| Direct `#[ignore]` count in `src/tests` | `75` |
| Live production potion fallback callsites | `0` |
| Direct relic helper-path refs | `0` |

Cleanup-only card shells:

- [reflex.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/silent/reflex.rs:1)
- [tactician.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/silent/tactician.rs:1)
- [deusexmachina.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/watcher/deusexmachina.rs:1)

All-content blockers still open:

- `Match and Keep!` minigame runtime
- `Liquid Memories` discard-choice fidelity
- `Smoke Bomb` back-attack positional legality

## Why The Branch Is Trusted

Representative green suites on the current audited tree:

- `./scripts/test_engine_rs.sh check --lib`
- `./scripts/test_engine_rs.sh test --lib --no-run`
- `test_run_parity` `19 passed`
- `test_rl_contract` `12 passed`
- `test_search_harness` `5 passed`
- `test_reward_runtime` `10 passed`
- `test_events_parity` `7 passed`
- `test_event_runtime_wave19` `3 passed`
- `test_event_runtime_wave20` `2 passed`
- `test_event_runtime_wave21` `2 passed`
- `test_potion_runtime_wave8` `6 passed, 1 ignored`
- `test_potion_runtime_action_path` `15 passed`
- `test_relic_runtime_wave17` `2 passed`
- `test_dead_system_cleanup_wave22` `1 passed`
- `test_generated_choice_java_wave3` `7 passed`
- `test_orb_runtime_java_wave1` `8 passed, 1 ignored`
- `test_card_runtime_watcher_wave26` `3 passed`

## Current Read

- If the claim is `supported runtime parity complete`, the branch is ready after cleanup/doc sync.
- If the claim is `all gameplay content complete`, do not mark the PR ready yet; close the three blockers above first.
