# Wave 1 Engine Fix Scorecard

**Judged engine commit:** `928381cc2d0804104c03c0c763ac87935436250c`

**Audit base:** `a43199b41c0f52374bda107beacfcc44dcb702a9`

**Judge date:** 2026-07-16

## Final gates

| Gate | Verdict | Evidence |
|---|---|---|
| G1 audit reproducers | PASS | `./scripts/test_engine_rs.sh test --lib test_audit_repros -- --nocapture`: `19 passed`, `0 failed`, `0 ignored`. Individual fix commits retain the base red and post-fix green evidence. |
| G2 full library | PASS | Post-final-behavior-commit run: `2927 passed`, `0 failed`, `0 ignored`. Baseline was `2883 passed`, `11 ignored`. |
| G3 smoke oracle | NEEDS REMINT | `/tmp` replay exited `1`: `status=diverged`, `matched_actions=1`, `total_actions=5`, first delta at action `1`, `post.rng.ai`. The protected golden selects `JawWorm`; the current TraceLab decimal seed contract and Java-derived weighted queue select `Small Slimes`. See the correction below. |
| G4 performance | PASS | `full_turn_cycle` upper bound `4.4541 us` (< `5.2 us`); `clone_for_mcts` `612.12 ns` (< `800 ns`); `get_legal_actions` `88.582 ns` (< `130 ns`). |
| G5 hygiene | PASS | `cargo check --all-targets` clean; zero ignored tests; zero protected-path changes; zero production raw card-effect reads; zero legacy card-effect registry symbols; clean worktree. |

## Queue verdicts

The shared G1 command below was rerun on the integrated final tree and exercised every `EDA-001` through `EDA-013` reproducer:

`./scripts/test_engine_rs.sh test --lib test_audit_repros -- --nocapture`

| Item | Verdict | Command / exit | Result |
|---|---|---|---|
| EDA-001 | FIXED-PROVEN | Shared G1 command / `0` | Victory dispatch completes all installed handlers. |
| EDA-002 | FIXED-PROVEN | Shared G1 command / `0` | Card `misc` and snapshot paths preserve Java `int` range. |
| EDA-003 | FIXED-PROVEN | Shared G1 command / `0` | Status amounts and Catalyst stacking preserve Java `int` range. |
| EDA-004 | FIXED-PROVEN | Shared G1 command / `0` | Room-reset streams use `seed + floor`; potion RNG remains persistent. |
| EDA-005 | FIXED-PROVEN | Shared G1 command / `0` | Seeded weighted encounter queues, exclusions, and repeat rules are active. |
| EDA-006 | FIXED-PROVEN | Shared G1 command / `0` | Nested runtime events retain the installed dispatch runtime. |
| EDA-007 | FIXED-PROVEN | Shared G1 command / `0` | Devotion enters Divinity without inventing Mantra remainder. |
| EDA-008 | FIXED-PROVEN | Shared G1 command / `0` | Necronomicon replays normal and X-cost attacks through card-use semantics. |
| EDA-009 | FIXED-PROVEN | Shared G1 command / `0` | Enemy Ritual uses owner round-end timing and skip-first state. |
| EDA-010 | FIXED-PROVEN | Shared G1 command / `0` | Static Discharge uses canonical channel/evoke/Electrodynamics behavior. |
| EDA-011 | FIXED-PROVEN | Shared G1 command / `0` | Final-enemy Spore Cloud no longer applies post-combat Vulnerable. |
| EDA-012 | FIXED-PROVEN | Shared G1 command / `0` | Combat shuffles use one outer RNG tick plus Java `Collections.shuffle`. |
| EDA-013 | FIXED-PROVEN | Shared G1 command / `0` | Player Poison ticks at owner turn start. |
| F2 | FIXED-PROVEN | `./scripts/test_engine_rs.sh test --lib run_trace_exposes_every_java_rng_counter_before_and_during_combat` / `0` | All 13 run RNG counters are emitted. |
| F4 | FIXED-PROVEN | Shared G1 command / `0` | Neow options map to Java blessing categories while intentionally exposing all four choices. |
| F5 | FIXED-PROVEN | `./scripts/test_engine_rs.sh test --lib trace_preserves_java_potion_slot_placeholders_outside_and_during_combat` / `0` | Empty potion slots emit `Potion Slot`. |
| Real relic counters | FIXED-PROVEN | `./scripts/test_engine_rs.sh test --lib trace_emits_real_relic_counters_from_run_and_combat_runtime_state` / `0` | Trace output no longer hardcodes relic counters. |
| EDA-019 | FIXED-PROVEN | `./scripts/test_engine_rs.sh test --lib note_for_yourself_profile_inputs_are_isolated_between_simulation_roots` / `0` | `NoteForYourself` profile state is root-local and emits explicit updates. |
| EDA-021 | NOT-ATTEMPTED | Not run | PyO3 feature isolation was deferred to the training/core-boundary design pass. |
| EDA-022 | NOT-ATTEMPTED | Not run | Additive-tolerant schema versioning was deferred with the boundary redesign. |

## Additional verified corrections

| Correction | Proof |
|---|---|
| Trace scripts now use TraceLab's decimal-first seed parsing. | `./scripts/test_engine_rs.sh test --lib script_seed_parsing_matches_tracelab_decimal_then_display_precedence` |
| `RandomXS128.nextInt(bound)` matches the shipped libGDX class. | `./scripts/test_engine_rs.sh test --lib random_xs128_bounded_ints_match_shipped_java_class` |
| Map path rerolls and Emerald Elite placement consume the Java map stream. | `./scripts/test_engine_rs.sh test --lib map_path_rerolls_match_java_counter_for_smoke_seed` |
| Inserter's turn counter persists across combats. | `./scripts/test_engine_rs.sh test --lib inserter_counter_persists_across_combats` |
| Runs continue through Act 2 and Act 3 instead of ending after the first boss relic. | `./scripts/test_engine_rs.sh test --lib boss_reward_screen_requires_relic_choice_and_transitions_to_act_two` and `second_boss_relic_transitions_to_act_three_with_a_fresh_map` |
| Ascension 20 uses the second shuffled Act 3 boss before the Spire Heart. | `./scripts/test_engine_rs.sh test --lib ascension_twenty_runs_the_second_act_three_boss_before_spire_heart` |

## Corrections to the register and oracle

### The smoke golden is stale under the checked-in harness contract

`smoke-neow-floor1.json` stores the decimal seed string `57554006466`. The current Java harness parses decimal strings directly in `TraceLabMod.parseSeed`, and the golden header also records `seed_long=57554006466`. The protected golden nevertheless enters a `JawWorm` combat, while replay under that decimal seed and the Java-derived weighted encounter queue enters `Small Slimes`.

The final `/tmp` comparison therefore reports its first difference at action `1`:

- golden: one `JawWorm`, `ai=1`, `monsterHp=1`, `misc=0`
- replay: `SpikeSlime_S` plus `AcidSlime_M`, `ai=2`, `monsterHp=2`, `misc=1`

The engine must not be changed back to a different seed interpretation or forced encounter to satisfy this artifact. A human-attended Java trace remint is required; no mask was added and no protected trace was edited.

### Later-act continuity was an unregistered simulation blocker

The audit queue did not include the fact that boss-relic completion ended the Rust run after Act 1. The final behavior commit now covers:

- Act 1 and Act 2 boss chest transitions
- Java act-transition healing and card-RNG counter boundaries
- Act-specific map seeds and encounter/boss regeneration
- ordinary Act 2/3 combats after floor 16
- Secret Portal's Act 3 boss floor
- Act 3 Victory Room / Spire Heart routing
- the Ascension 20 second Act 3 boss

## Handoff

1. **Remint the smoke golden with the current checked-in TraceLab harness.** This is the only failed mechanical gate and is blocked on human Java execution, not an accepted Rust mismatch.
2. **Replace the simplified Act 4 chain with the real map sequence.** The current engine goes from Spire Heart acceptance directly to Shield/Spear and then Heart; the Act 4 rest site, shop, room transitions, floor/RNG resets, and key-removal presentation still need source-derived run-path proof.
3. **Start the training boundary redesign from this clean engine tip.** Prioritize EDA-014 causal snapshots, EDA-017 batched deterministic search, and EDA-021/022 core/PyO3/schema separation before coupling the new logging and distributed training system.

## Readiness verdict

The branch is a clean, zero-skip, performance-safe engine correction stack and is suitable as the base for the next training architecture branch. It is not an honest claim of complete full-run parity until the smoke trace is reminted and the remaining Act 4 sequence is implemented and traced.
