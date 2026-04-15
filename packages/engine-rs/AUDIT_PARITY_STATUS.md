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
- all-content gameplay parity: `99%`
- architecture unification snapshot: `99%`

Area scores:

| Area | Score | Notes |
| --- | ---: | --- |
| Combat runtime parity | `99%` | Public gameplay-gap card tail is closed and the former Defect multi-hit / X-cost blockers are green on engine-path suites |
| RL combat surface | `98%` | `Neow`, reward screen, decision context, and search surfaces are green |
| Run / reward / event parity | `99%` | `NoteForYourself`, `Match and Keep!`, and `Scrap Ooze` now run on canonical event/runtime paths |
| Dead-system retirement | `98%` | Helper-path production debt is effectively gone |

## Current Quantified Backlog

| Metric | Value |
| --- | ---: |
| Raw public card files with empty `effect_data` | `3` |
| Raw public card files with `complex_hook: Some(...)` | `0` |
| Unresolved public gameplay-gap files | `0` |
| Cleanup-only card shells | `3` |
| Blocked supported event ops | `0` |
| Explicit blocked event branches in source | `0` |
| Direct `#[ignore]` count in `src/tests` | `73` |
| Live production potion fallback callsites | `0` |
| Direct relic helper-path refs | `0` |

Cleanup-only card shells:

- [reflex.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/silent/reflex.rs:1)
- [tactician.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/silent/tactician.rs:1)
- [deusexmachina.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/cards/watcher/deusexmachina.rs:1)

Known gameplay blockers still open:

- none currently confirmed on the integrated branch

Remaining merge-readiness work:

- stale ignored-test cleanup and reclassification
- one final full Java audit sweep after the latest landed fixes

Major stale/noisy debt still present:

- watcher ignored placeholders for `Collect`, `Conjure Blade`, `Fasting`, `Judgement`, `Pressure Points`, `Wallop`, `Brilliance`, `Halt`, `Perseverance`, `Sands of Time`, and `Windmill Strike` now overstate the live parity tail

## Why The Branch Is Trusted

Representative green suites on the current audited tree:

- `./scripts/test_engine_rs.sh check --lib`
- `./scripts/test_engine_rs.sh test --lib --no-run`
- `test_run_parity` `19 passed`
- `test_rl_contract` `12 passed`
- `test_search_harness` `5 passed`
- `test_reward_runtime` `10 passed`
- `test_events_parity` `7 passed`
- `test_event_runtime_wave19` `6 passed`
- `test_event_runtime_wave20` `3 passed`
- `test_event_runtime_wave21` `2 passed`
- `test_potion_runtime_wave8` `8 passed`
- `test_potion_runtime_action_path` `15 passed`
- `test_relic_runtime_wave17` `2 passed`
- `test_dead_system_cleanup_wave22` `1 passed`
- `test_generated_choice_java_wave3` `7 passed`
- `test_orb_runtime_java_wave1` `9 passed`
- `test_card_runtime_defect_wave12` `7 passed`
- `test_card_runtime_xcount_wave2` `3 passed`
- `test_card_runtime_defect_wave9` `3 passed`
- `test_card_runtime_watcher_wave26` `3 passed`

## Current Read

- If the claim is `supported runtime parity complete`, the branch is ready after cleanup/doc sync.
- If the claim is `all gameplay content complete`, the known gameplay blockers are closed, but the PR still needs one final audit and ignored-test cleanup before we call it ready.
- Zero-skip answer: `no` — there are still `73` explicit `#[ignore]` tests in `src/tests`.
- Java-clean answer: `not yet formally re-frozen` — the known gameplay blockers are closed, but the final audit sweep still needs to refresh the branch-wide claim.
