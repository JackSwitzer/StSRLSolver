# Audit 5 — Test-suite drift Cycles 2-6

**Status:** REOPEN (one silent bug-close; 43 pre-existing orphans documented)
**Total Rust tests:** 2327 passed / 0 failed / 10 ignored (matches Cycle 1 baseline)

## Weak-assert hunt

No NEW weak asserts introduced in Cycles 2-6. All 12 grep hits for `assert!(x > 0)` / `assert!(x >= 0)` / `all(.. cost == 0)` patterns are pre-existing in non-enemy files (entity_runtime, search_harness, run_parity, dead_system_cleanup_wave5, relic_runtime_wave9, card_runtime_defect_wave2, card_runtime_generated_choice_wave6, card_runtime_temp_wave1-equivalent, card_runtime_silent_wave1-equivalent, power_runtime_metadata_wave1). `test_enemies.rs` and `test_bosses.rs` — the Cycle 1 targets — remain clean (all tight `assert_eq!`).

- `test_power_runtime_metadata_wave1.rs:24` — `assert!(status(DEMON_FORM) > 0)` is weak (could be 0, 1, 99); pre-existing, out of Cycle 1 scope.

## Duplicate coverage

None. `test_enemies.rs:339` and `:438` retain the Cycle 1 comments `"Guardian / Hexaghost / SlimeBoss boss tests moved to test_bosses.rs"` and `"BronzeAutomaton / AwakenedOne / CorruptHeart boss tests moved to..."`. Grep for boss names in `test_enemies.rs` returned only those two comment lines — no function bodies. Cycle 4a's `test_corruptheart_a0.rs` is a NEW file testing D143/D144-specific behavior, not a duplicate of `test_bosses.rs`.

## Fresh-RNG leaks

None outside the known RNG-testing paths. All 51 `StsRandom::new(0)` occurrences live in `test_enemies.rs` and `test_bosses.rs`, which are explicit enemy-AI roll-testing files. Cycle 1's fix to `roll_next_move_with_num` was NOT regressed — zero usages of the old fresh-RNG pattern in `test_corruptheart_a0.rs`, `test_combat_hooks_integration.rs`, or `test_stage_b_promotions.rs`. Cycle 6's Stage B tests correctly use `roll_next_move_with_num(&mut e, N)`.

## Ignored-test catalog

All 10 `#[ignore]` tests in `test_stage_b_promotions.rs` have a D-# tag, a Java file:line reference, and a fix-cycle annotation (`"Dxxx open — Cycle 8+"`). File header at line 9-12 cross-refs `docs/work_units/parity-deviations-register.md` and decompiled Java paths.

- `d172_collector_a19_scaling` — D172, Java `TheCollector.java:97`, FAILS as ignored (bug open, correct)
- `d173_collector_revive_dispatch` — D173, Java `TheCollector.java:189-192`, FAILS as ignored (correct)
- `d174_damaru_timing_java_is_pre_draw` — D174, Java `Damaru.java:32`, FAILS as ignored (correct)
- `d175_devotion_timing_java_is_post_draw` — D175, Java `DevotionPower.java:34`, FAILS as ignored (correct)
- `d176_scry_has_on_scry_trigger_variant` — D176, Java `ScryAction.java:37-40`, PANICS as ignored (correct)
- `d177_electrodynamics_bypasses_enemy_block` — D177, Java `LightningOrbPassiveAction.java:56`, FAILS as ignored (correct)
- `d178_distilled_chaos_random_target_per_card` — D178, Java `DistilledChaosPotion.java:36-41`, PANICS as ignored (correct)
- **`d179_plated_armor_not_decremented_by_thorns_or_hp_loss`** — D179, Java `PlatedArmorPower.java:55`, **PASSES as ignored** → bug silently closed, `#[ignore]` must be removed
- `d180_wraith_form_java_is_end_of_turn_and_stack_scaled` — D180, Java `WraithFormPower.java:33`, FAILS as ignored (correct)
- `d181_council_of_ghosts_scales_with_max_hp_and_ascension` — D181, Java `Ghosts.java:36-38`, FAILS as ignored (correct)

**REOPEN finding**: D179 `d179_plated_armor_not_decremented_by_thorns_or_hp_loss` passes when `--ignored` is forced. Either the test doesn't actually exercise the broken path (trigger gating/damage-type gate was silently wired somewhere in Cycles 2-6, or the test as written runs through a branch that never decrements), or the bug was fixed without deleting the ignore. Before merge: remove `#[ignore]`, confirm it stays green, and close D179 in the deviations register — or strengthen the test so it actually fails against the documented Rust path (`combat_hooks.rs:186-191`).

## Orphan files

**43 orphan test files in `packages/engine-rs/src/tests/*.rs` that are NOT registered in `mod.rs`.** All pre-date Cycle 1 (git blame traces them to `03611329`, `fbbf84f4`, and older). Cycles 2-6 added 5 mod lines (corruptheart_a0, stage_b_promotions, combat_hooks_integration, powers_dispatch_wired, runtime_inline_cutover_wave2) but did not re-introduce new orphans. Still worth flagging since ~43 files of `#[test]` bodies compile-to-nothing.

Full orphan list: `test_card_runtime_backend_wave3`, `test_card_runtime_defect_wave{3,4,5,6,7}`, `test_card_runtime_ironclad_wave{3,4,6,7}`, `test_card_runtime_silent_wave{3,4,7}`, `test_card_runtime_watcher_wave{3,4,5,6}`, `test_damage_followup_java_wave1`, `test_defect_java_wave1`, `test_event_runtime_wave{4,5,6,8,9,10,11,13}`, `test_generated_choice_java_wave3`, `test_orb_runtime_java_wave1`, `test_played_card_instance_state`, `test_potion_runtime_action_path`, `test_potion_runtime_wave{4,5,6,8}`, `test_power_runtime_{card_play,complex,debuff_enemy,end_to_end,replay,turn_start}`, `test_runtime_inline_cutover_wave{4,5}`, `test_zone_batch_java_wave2`.

## Recommendation

**BLOCK — remediation before merge is trivial:**

1. **D179 ignore removal** (1-line fix in `test_stage_b_promotions.rs:290`): delete `#[ignore = "D179 open — Cycle 8+"]`, confirm green, close D179 in the deviations register. If the test passes because of test shape rather than a real fix, strengthen the scenario to hit `combat_hooks.rs:186-191` directly before unignoring.

2. **Orphan files**: out-of-scope for this merge (pre-existing). File a follow-up to either register them in `mod.rs` or delete them — do not gate merge on this.

With (1) resolved, the suite is clean of Cycle 2-6 drift.
