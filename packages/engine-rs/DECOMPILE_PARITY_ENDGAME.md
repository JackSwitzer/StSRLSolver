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
- `Scrap Ooze` now resolves through the canonical event reward runtime
- `NoteForYourself` now runs as a real two-step shrine with cross-run card stash behavior inside the runtime
- `Stance Potion` is fully green and `Smoke Bomb` legality/action-path behavior is green except for the explicit Java positional `BackAttack` caveat
- the `Scrawl+` hand-limit and `Deus Ex Machina+` draw-order edge cases now have explicit engine-path proof

Live branch truth:

| Metric | Value |
| --- | ---: |
| Raw public empty `effect_data` card files | `3` |
| Raw public `complex_hook` card files | `0` |
| Unresolved public gameplay-gap files | `0` |
| Blocked supported event ops | `0` |
| Explicit blocked event branches | `1` |
| Direct ignored tests | `74` |

The raw empty public-card files are cleanup shells only:

- `Reflex`
- `Tactician`
- `Deus Ex Machina`

## What Still Blocks Full All-Content Parity

These are the only meaningful remaining gameplay families:

1. `Match and Keep!`
   - still explicitly blocked
   - needs a real GremlinMatchGame-style card-grid runtime
   - current blocker proof lives in [test_event_runtime_wave19.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_event_runtime_wave19.rs:1)

2. `Scrap Ooze`
   - current runtime only models the first-success damage-plus-relic branch
   - Java has a retry loop with escalating damage, escalating relic chance, and an explicit flee branch
   - current simplified proof lives in [test_event_runtime_wave20.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_event_runtime_wave20.rs:1)

3. Defect multi-hit family
   - the typed `ExtraHits(...)` path currently clamps to at least one hit
   - Java does zero hits when Barrage has no orb slots and when Thunder Strike has zero Lightning channeled
   - `Target::RandomEnemy` is currently rolled once per card instead of once per hit, so `Rip and Tear` and `Thunder Strike` reuse a single target
   - current blocker proof lives in [test_card_runtime_defect_wave12.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_card_runtime_defect_wave12.rs:1)

4. `Reinforced Body`
   - current Rust typing still models one block gain instead of Java repeated-block/X-cost resolution
   - current blocker proof lives in [test_card_runtime_defect_wave9.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_card_runtime_defect_wave9.rs:1)

5. `Smoke Bomb` positional legality
   - boss legality and regular flee behavior are correct
   - Java `BackAttack` / Surrounded caveat still needs positional combat state
   - current blocker proof lives in [test_potion_runtime_wave8.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_potion_runtime_wave8.rs:218)

## Immediate Execution Order

If the goal is to leave draft only after `all gameplay content complete`, the next implementation order should be:

1. `Match and Keep!` minigame runtime
2. `Scrap Ooze` retry/flee state machine
3. Defect multi-hit fix for `Barrage` / `Rip and Tear` / `Thunder Strike`
4. repeated-block / X-cost primitive for `Reinforced Body`
5. positional legality state for `Smoke Bomb`
6. ignored-test cleanup pass
7. final audit refresh and PR readiness sweep

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
