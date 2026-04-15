# Decompile-Backed Parity Endgame

Last updated: 2026-04-14  
Branch: `codex/universal-gameplay-runtime`

Java oracle root:

- `/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src`

Canonical audit docs:

- [INCONSISTENCY_REPORT.md](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/INCONSISTENCY_REPORT.md:1)
- [AUDIT_PARITY_STATUS.md](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/AUDIT_PARITY_STATUS.md:1)
- [DESIGN_DECISIONS.md](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/DESIGN_DECISIONS.md:1)

## Current Checkpoint

What closed in the latest pass:

- `Emotion Chip` now pulses on the following turn start instead of firing immediately on HP loss
- `Liquid Memories` now opens a real discard choice, supports Sacred Bark multi-pick, and returns selected cards at zero cost
- `Match and Keep!` now runs as a real indexed match/minigame loop on the canonical event runtime
- `Scrap Ooze` now runs the Java-style retry / flee / escalating relic-chance loop on the canonical event runtime
- Defect multi-hit parity is now closed for `Barrage`, `Rip and Tear`, and `Thunder Strike`
- `Reinforced Body` already matches the typed X-cost block surface and the stale blocker is removed
- `NoteForYourself` now runs as a real two-step shrine with cross-run card stash behavior inside the runtime
- `Stance Potion` and `Smoke Bomb` legality/action-path behavior are fully green
- the `Scrawl+` hand-limit and `Deus Ex Machina+` draw-order edge cases now have explicit engine-path proof

Live branch truth:

| Metric | Value |
| --- | ---: |
| Raw public empty `effect_data` card files | `3` |
| Raw public `complex_hook` card files | `0` |
| Unresolved public gameplay-gap files | `0` |
| Blocked supported event ops | `0` |
| Explicit blocked event branches | `0` |
| Direct ignored tests | `69` |

The raw empty public-card files are cleanup shells only:

- `Reflex`
- `Tactician`
- `Deus Ex Machina`

## What Still Blocks Full All-Content Parity

The final audit turned up one confirmed active mismatch plus a small ignored-family tail:

1. `Establishment` retained-card cost reduction still misses the Java end-of-turn cost drop
2. ignored-family tail still awaiting fresh green proof: `Headbutt`, `Violence`, `Secret Technique` legality, `Mind Blast`, `Pressure Points`, `Sentinel` under `Corruption`
3. stale ignored-test cleanup and reclassification
4. one more full Java audit freeze after the audit-hygiene cleanup

## Immediate Execution Order

If the goal is to leave draft only after `all gameplay content complete`, the next implementation order should be:

1. fix `Establishment`
2. resolve or explicitly reclassify the small ignored-family tail
3. ignored-test cleanup pass
4. final audit refresh and PR readiness sweep
5. training branch cut from this branch

If the claim stays `supported runtime parity complete`, the next order should instead be:

1. docs / PR sync
2. ignored-test cleanup
3. training branch cut from this branch

## Verification Substrate

Worker acceptance remains:

- `./scripts/test_engine_rs.sh check --lib`
- `./scripts/test_engine_rs.sh test --lib --no-run`
- focused engine-path suites for the owned slice

Representative currently green suites:

- `test_run_parity`
- `test_rl_contract`
- `test_search_harness`
- `test_reward_runtime`
- `test_events_parity`
- `test_event_runtime_wave19`
- `test_event_runtime_wave20`
- `test_event_runtime_wave21`
- `test_card_runtime_defect_wave12`
- `test_card_runtime_xcount_wave2`
- `test_potion_runtime_wave8`
- `test_potion_runtime_action_path`
- `test_relic_runtime_wave17`
- `test_dead_system_cleanup_wave22`
- `test_generated_choice_java_wave3`
- `test_orb_runtime_java_wave1`
- `test_card_runtime_watcher_wave26`

## Audit-Confirmed Stale Noise

The latest partitioned Java audit also found a meaningful cleanup tail that should not be counted as live gameplay gaps:

- stale watcher ignored placeholders for `Collect`, `Conjure Blade`, `Fasting`
- stale watcher ignored placeholders for `Judgement`, `Pressure Points`, and `Wallop`
- stale watcher ignored placeholders for `Brilliance`, `Halt`, `Perseverance`, `Sands of Time`, and `Windmill Strike`
- these are already covered by later engine-path suites and should be removed in the next cleanup wave
