# Decompile-Backed Parity Endgame

Last updated: 2026-04-15  
Branch: `codex/universal-gameplay-runtime`

Java oracle root:

- `/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src`

Canonical audit docs:

- [INCONSISTENCY_REPORT.md](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/INCONSISTENCY_REPORT.md:1)
- [AUDIT_PARITY_STATUS.md](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/AUDIT_PARITY_STATUS.md:1)
- [DESIGN_DECISIONS.md](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/DESIGN_DECISIONS.md:1)

## Current Checkpoint

What closed in the latest cleanup pass:

- `Establishment` now applies retained-card combat cost reduction correctly across turns
- stale watcher/colorless/defect ignore noise was converted into real passing coverage
- stale active assertions were fixed for `Consecrate`, `Purity`, and `Capacitor`
- `Secret Technique` legality is now proven on the engine path instead of staying ignored
- `DiscoveryAction`, `Chrysalis`, and `Metamorphosis` are now covered by real passing generated-choice tests instead of ignored placeholders

Live branch truth:

| Metric | Value |
| --- | ---: |
| Raw public empty `effect_data` card files | `3` |
| Raw public `complex_hook` card files | `0` |
| Unresolved public gameplay-gap files | `0` |
| Blocked supported event ops | `0` |
| Explicit blocked event branches | `0` |
| Direct ignored tests | `5` |

The raw empty public-card files are cleanup shells only:

- `Reflex`
- `Tactician`
- `Deus Ex Machina`

## What Still Blocks Full All-Content Parity

The real remaining gameplay tail is now:

1. `Parasite` master-deck removal max-HP semantics
2. `Sentinel` under `Corruption`
3. `Expunger` temp-card X-count / repeated-hit state fidelity

Cleanup-only remaining ignores:

1. relic bridge retirement in `test_dead_system_cleanup_wave18`
2. relic bridge retirement in `test_dead_system_cleanup_wave19`

## Immediate Execution Order

If the goal is to leave draft only after `all gameplay content complete`, the next implementation order should be:

1. exhaust / temp-card family
   - `Sentinel` under `Corruption`
   - `Expunger`
2. relic / curse follow-up family
   - `Parasite`
3. dead-system bridge retirement
4. final audit refresh and PR readiness sweep
5. training branch cut from this branch

If the claim stays `supported runtime parity complete`, the next order should instead be:

1. docs / PR sync
2. training branch cut from this branch
3. close the remaining all-content tail on follow-up work

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

## Audit-Confirmed Cleanup Result

This pass removed the misleading “big unknown” feel from the tail:

- direct ignored tests dropped from `69` to `5`
- generated-choice fidelity for `DiscoveryAction`, `Chrysalis`, and `Metamorphosis` is now covered by real passing tests
- stale watcher placeholders for `Collect`, `Conjure Blade`, `Fasting`, `Judgement`, `Pressure Points`, `Wallop`, `Brilliance`, `Halt`, `Perseverance`, `Sands of Time`, and `Windmill Strike` are no longer overstating parity debt
- stale colorless/choice placeholders for `Headbutt`, `Violence`, and `Secret Technique` legality are gone
- the parity PR can now talk about a short explicit tail instead of a broad fuzzy backlog
