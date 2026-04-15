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
- `Parasite` now applies Java-style max-HP loss only when removed from the master deck
- `Sentinel` under `Corruption` now refunds energy through a typed exhaust hook lane
- `Expunger` / `Conjure Blade+` now preserve card-owned X-count semantics, including `Chemical X`
- registry-backed secondary behavior now runs through typed runtime-trigger metadata instead of raw registry/tag ownership
- production/runtime/export code now has `0` raw `card.effects` reads and `0` live registry-dispatch symbols on the verified source audit
- gameplay export now emits structured runtime traits/triggers/play hints instead of semantic effect tags
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
| Direct ignored tests | `0` |
| Final broad freeze | `2188 / 2188` |

The raw empty public-card files are intentional runtime-trigger-only cards:

- `Reflex`
- `Tactician`
- `Deus Ex Machina`

Those files are no longer semantic outliers: their live runtime behavior comes from the typed runtime-trigger surface, and `test_card_runtime_nonplay_triggers_wave1` now proves them as explicit runtime-only defs.

## Last Known Blocker Sweep

The last explicit semantic blocker sweep is now closed:

1. `Collect` now resolves Miracles before draw and is covered by [test_card_runtime_watcher_wave24.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_card_runtime_watcher_wave24.rs:154)
2. free-play X-cost cards now preserve energy while snapshotting X and are covered by [test_card_runtime_xcount_wave1.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_card_runtime_xcount_wave1.rs:105)
3. `Emotion Chip` now uses full multi-orb `ImpulseAction`-style replay, including `Cables`, in [test_orb_runtime_java_wave1.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_orb_runtime_java_wave1.rs:239)
4. shop purge pricing now persists across visits and discounts correctly in [test_run_parity.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_run_parity.rs:91)

## Immediate Execution Order

The gameplay-parity side is now through the endgame gate:

1. training branch cut from this branch
2. keep broad confidence audits running as non-blocking verification work
3. treat any further authoring cleanup as optional follow-on polish rather than parity work

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
- `test_card_runtime_watcher_wave24`
- `test_card_runtime_xcount_wave1`
- `test_runtime_inline_cutover_wave5`
- `test_card_runtime_nonplay_triggers_wave1`
- `test_card_runtime_support_wave1`
- `test_cards_ironclad`
- `test_cards_defect`
- `test_cards_silent`
- `test_cards_watcher`
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

- direct ignored tests dropped from `69` to `0`
- generated-choice fidelity for `DiscoveryAction`, `Chrysalis`, and `Metamorphosis` is now covered by real passing tests
- stale watcher placeholders for `Collect`, `Conjure Blade`, `Fasting`, `Judgement`, `Pressure Points`, `Wallop`, `Brilliance`, `Halt`, `Perseverance`, `Sands of Time`, and `Windmill Strike` are no longer overstating parity debt
- stale colorless/choice placeholders for `Headbutt`, `Violence`, and `Secret Technique` legality are gone
- the parity PR can now talk about a zero-skip audited matrix with a completed `2188 / 2188` broad freeze instead of a broad fuzzy backlog
