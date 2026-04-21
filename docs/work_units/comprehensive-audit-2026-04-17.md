# Comprehensive Audit 2026-04-17

Branch: `claude/sharp-solomon-a1c9ec` (training-rebuild stack)
Trigger: recorded-run replay of `runs/WATCHER/1776347657.run` (23 combats; 5 solved, 0 failed, 18 unsupported, 0 error). Report: `logs/active/recorded-run-20260417-220718/recorded_run_replay_report.json`.

This is the master plan for the next several work sessions. All §1 bugs are confirmed by source citations. All §5 tests are scoped to file paths. Read in order.

## §1 Confirmed parity bugs

### 1.1 Enemy AI consumes no RNG (deterministic intent)

- Symptom: every enemy in every combat picks the most-common Java branch deterministically. Probabilistic intent variance is absent. Multi-enemy combats lose Java's shared `aiRng` stream order coupling.
- Smoking gun:
  - Java: `decompiled/java-src/com/megacrit/cardcrawl/monsters/AbstractMonster.java:465-466` — `rollMove() { this.getMove(AbstractDungeon.aiRng.random(99)); }`. 25 monster files use `aiRng.randomBoolean(...)` inside `getMove(int num)` (e.g. `monsters/exordium/JawWorm.java:152-180`, `monsters/exordium/AcidSlime_M.java:108-159`).
  - Rust: `packages/engine-rs/src/enemies/mod.rs:815` — `pub fn roll_next_move(enemy: &mut EnemyCombatState)` takes no RNG. `packages/engine-rs/src/enemies/act1.rs:11-21` `roll_jaw_worm` uses only `last_move`/`last_two_moves` chains, no `aiRng.random(99)` seed and no `randomBoolean(...)` branches. `packages/engine-rs/src/state.rs:142-156` `EnemyCombatState` has no rng field. The CombatEngine's own RNG (`packages/engine-rs/src/engine.rs:118` — `pub(crate) rng: crate::seed::StsRandom`) exists but is never threaded into `roll_next_move`. Single call site is `packages/engine-rs/src/combat_hooks.rs:511` — `enemies::roll_next_move(&mut engine.state.enemies[enemy_idx])`.
- Impact: any combat with branching intent diverges from Java. Confirmed Watcher-relevant impact: JawWorm post-Bellow branching, Cultist (only one move so safe), AcidSlime_M/L on every turn after the first, Sentry (multi-enemy stream order), Looter, GremlinNob's 1/3 vs 2/3 Bellow split, Lagavulin's wake choice, Hexaghost subroutines. Confirmed Acts-2-3 impact: Chosen, BronzeAutomaton, Centurion+Healer pair ordering, Reptomancer, GremlinLeader. Tonight's "solver vs human" delta on JawWorm (0.39 vs 6) and Guardian-class fights cannot be disentangled until this is fixed.
- Fix scope (structural, ~5–10 files, ~200+ LOC):
  1. Add `&mut StsRandom` argument to `roll_next_move` and every `roll_*` in `enemies/{mod,act1,act2,act3,act4}.rs`.
  2. At top of `roll_next_move`, draw `let num = rng.random(99);` (Java parity) and pass `num` plus `rng` down to per-enemy fns.
  3. Re-implement probabilistic branches per Java: at minimum JawWorm (4 randomBoolean calls), AcidSlime_M/L, GremlinNob, Lagavulin, Hexaghost subroutines, Chosen, Sentry pair selection, BronzeAutomaton, Centurion+Healer, GremlinLeader, Reptomancer, ShelledParasite, Byrd, Mystic, BookOfStabbing, TaskMaster, Darkling, Mugger, WrithingMass, Nemesis, Champ, SphericGuardian, Healer, Maw, GiantHead, OrbWalker, Repulsor, Transient, SpireSpear, SpireShield, CorruptHeart, TimeEater. Java reference under `decompiled/java-src/com/megacrit/cardcrawl/monsters/`.
  4. Update `combat_hooks.rs:511` call site to pass `&mut engine.rng`.
  5. Add `first_turn` semantics: Java's `getMove` checks `firstMove` before consuming `num` for many enemies; mirror that. `EnemyCombatState.first_turn` exists at `state.rs:154` but is currently unused for AI gating.
- Test that would have caught it: see §5.1 (`test_jaw_worm_rollmove_consumes_rng`).

### 1.2 Solver reports `expected_hp_loss` from incomplete rollouts, not actual played-out combat

- Symptom: per-combat report shows fractional HP losses like `solver_hp_loss: 0.733` against a `recorded_hp_loss: 2` integer. HP loss in this game is always integer; fractional values mean the value is a Monte Carlo average over rollouts that never reached terminal under the search budget.
- Smoking gun:
  - `packages/training/run_replay.py:239` — `solver_hp_loss = parsed.root_outcome.expected_hp_loss`. This reads the root node's averaged outcome vector, not an actual integer HP delta from a played-out combat.
  - `packages/engine-rs/src/search.rs:759-768` — `terminal_outcome` is well-defined for terminal states, but at line 752 (`puct_score`) and lines 125-149 (the `expected_hp_loss` accumulator) the value is averaged across all child rollouts. Most rollouts hit `TimeCap` (every result in tonight's report has `stop_reason: "TimeCap"`), so `expected_hp_loss` mixes terminal HP with leaf-evaluator value estimates.
  - Even when terminal: `terminal_outcome` reads `engine.state.player.max_hp - engine.state.player.hp` of the rollout, not the entry-vs-exit HP of the combat as scored against `recorded_damage_taken`.
- Impact: every "solved" line in the recorded-run report is impossible to compare integer-to-integer with the human's recorded HP loss. The pass criterion (`run_replay.py:8` — `solver_hp_loss <= recorded_hp_loss + max(base_tol, 0.1 * max_hp)`) silently treats the value-head estimate as a real HP outcome. The Jaw Worm "0.39 vs 6" delta is meaningless without per-turn replay producing an integer.
- Fix scope (multi-file, ~150 LOC):
  1. After PUCT root selection, do a per-turn replanning loop: re-run search at every player decision, take the chosen action, step the actual `RustCombatEngine`, repeat until `is_combat_over()`. Track real entry HP and real exit HP.
  2. Either expose this as a Rust helper on `CombatSolver` (preferred — keeps Python thin) or implement in `run_replay.py` using `solver.step(action_id)` after `solver.run_combat_puct(...)`. Files: `packages/engine-rs/src/lib.rs:158-307` (CombatSolver), `packages/training/run_replay.py:213-274` (replay loop).
  3. Stochastic enemies: Once §1.1 lands, run multiple replays per combat with different aiRng seeds and report mean+stddev integer HP loss instead of one number.
- Test that would have caught it: see §5.2 (`test_solver_hp_loss_is_integer_for_known_seed`).

### 1.3 PUCT search stops at `TimeCap` not `Converged` for every recorded combat

- Symptom: every result in tonight's report has `stop_reason: "TimeCap"`. Search is timing out, not converging, even on single-enemy hallway combats with only ~1700 visits used out of 4096 hard cap. User wants "search until matches or beats human" — current loop has no such target and no iterative deepening.
- Smoking gun:
  - `packages/engine-rs/src/search.rs:328-399` — main loop: TimeCap check first (line 338), HardVisitCap check (line 351), then convergence check (lines 373-398). Convergence requires `stable_windows_required` (default 3) consecutive convergence windows where best-action stability AND value delta AND visit-share lead all hold. With visit_window=256 and min_visits=1024, hitting 3 windows requires at minimum 1024+3*256 = 1792 visits with no oscillation. Hallway preset time_cap_ms=1500 (`training_contract.rs:359`) routinely runs out before convergence.
  - `packages/training/stage2_pipeline.py:481-501` `_config_for_room` is the only knob; it is room-keyed not goal-keyed. There is no "search until solver_hp_loss <= recorded_hp_loss" loop anywhere in the stack.
- Impact: search budget is insufficient for stable best-action selection, so the reported solver action sequence is noisy. Combined with §1.2, the "5 solved" claim from tonight is structurally weak: it is "5 combats where a noisy value estimate happened to fall under tolerance".
- Fix scope (single-session, ~100 LOC):
  1. Add an iterative-deepening helper around `search_combat_puct`: start with hallway preset, double `time_cap_ms` and `hard_visit_cap` until either `stop_reason == Converged` OR a goal predicate (e.g. `expected_hp_loss <= target`) is satisfied OR a hard wall (e.g. 60s for hallway, 300s for boss) is hit. Add to `packages/engine-rs/src/search.rs`.
  2. Plumb a `target_hp_loss: Option<f32>` field through `CombatPuctConfigV1` (`training_contract.rs:338`) so callers can ask for goal-conditioned termination.
  3. Update `run_replay.py` to pass `target_hp_loss = recorded_damage_taken` so the solver knows what to beat, and report when it gives up vs converges.
- Test: see §5.3 (`test_iterative_deepening_reaches_converged_or_goal`).

### 1.4 Energy hardcoded to 3 in recorded-run replay

- Symptom: `packages/training/run_replay.py:217` constructs `RustCombatEngine(..., 3, ...)` — energy is always 3.
- Impact: Watcher A0 baseline is 3 energy and most combats start with 3, but Mark of the Bloom, Runic Dome, and other relics modify starting energy. For runs containing those relics (or the Coffee Dripper → no rest interaction), entry state is wrong. Currently invisible because tonight's run has neither.
- Fix scope (single-line + 1 helper): derive energy from relic set. Pull from `RustCombatEngine` itself if it computes start-of-combat energy from relic state, or whitelist the modifiers in Python. ~20 LOC.
- Test: see §5.4 (`test_energy_modifier_relics_change_entry_state`).

### 1.5 Neow `TEN_PERCENT_HP_LOSS` cost not applied to floor-1 entry HP

- Symptom: `packages/training/run_parser.py:283-287` has explicit TODO — Neow cost for `TEN_PERCENT_HP_LOSS` does not adjust `current_hp` before the loop. Floor 1 entry HP equals `starting_max_hp` not `0.9 * max_hp`.
- Impact: floor-1 combat entry state is wrong by ~6 HP for runs that took the 10% HP cost Neow option. Tonight's run did not take that option so it didn't surface.
- Fix scope (single-line in `_apply_neow` + `reconstruct_combat_cases`): apply the HP loss before constructing the floor-1 case.
- Test: see §5.5 (`test_neow_ten_percent_hp_loss_drops_entry_hp`).

### 1.6 PUCT uses MLX evaluator on every leaf with no batching

- Symptom: `packages/training/engine_adapter.py` `build_model_evaluator` returns a closure called per leaf. Every leaf call is a single-sample MLX inference round trip.
- Smoking gun: `packages/engine-rs/src/search.rs:280-400` calls `evaluator(&CombatTrainingStateV1)` per simulate cycle (line 333). `packages/training/inference_service.py` `CombatInferenceService` does not batch across leaves of the same root.
- Impact: TimeCap (§1.3) is hit far earlier than necessary because each visit waits on a synchronous Python→MLX→Python round trip. With ~1700 visits in 1.5s, that is sub-1ms per evaluation, so the bottleneck may currently be Rust↔Python crossings not MLX itself, but either way batching would help.
- Fix scope (multi-session, ~300 LOC, requires search.rs structural change to enqueue+yield instead of evaluator-per-leaf): out of scope for the next sprint, but list now so it's tracked.

### 1.7 `TIME_PROBABLY_FORWARD` Neow bonuses unhandled in deck reconstruction

- Symptom: `_apply_neow` (`run_parser.py:347-378`) handles only `REMOVE_TWO`, `ONE_RANDOM_RARE_CARD`, `BOSS_RELIC` explicitly; comment claims "Other Neow bonuses (HUNDRED_GOLD, MAX_HP, REMOVE_CARD, etc.) do not affect deck" but several do affect deck (e.g. `THREE_CARDS` adds 3 cards to choose from, `TRANSFORM_CARD`, `UPGRADE_CARD`, `ONE_RARE_RELIC` adds a relic). Most of these would fail `_validate_final_deck` silently with a warning rather than error.
- Impact: silent reconstruction errors for some Neow paths. Final-deck validator will catch the big ones.
- Fix scope (single-session, ~50 LOC): add explicit handlers for `TRANSFORM_CARD`, `UPGRADE_CARD`, `THREE_CARDS`, `ONE_RARE_RELIC`. The `.run` schema records the chosen card for `TRANSFORM_CARD` / `UPGRADE_CARD` in `event_choices`/`card_choices` so this is mostly bookkeeping.
- Test: see §5.6 (`test_neow_transform_upgrade_threecards_reconstruction`).

## §2 Approach issues (process / design gaps)

### 2.1 Encounter catalog Acts 2-4 missing in Python

- Python `packages/training/encounters.py:31-150` defines 16 encounter entries — all Act 1. Rust `packages/engine-rs/src/enemies/mod.rs:22-64` registers 38 enemy IDs spanning all four acts (Cultist, Chosen, Mugger, Byrd, Centurion, Mystic, Reptomancer, Nemesis, Transient, Maw, SpireGrowth, SpireShield, SpireSpear, etc.). The gap is Python wrapping.
- Impact: drove 18/23 combats in tonight's recorded-run replay to `unsupported` status. The engine could have run all 18; the wrapping just isn't there.
- Fix scope (single-session, ~200 LOC of dataclass entries): port Java `monsters/city/MonsterHelper.java` and `monsters/beyond/MonsterHelper.java` group definitions into Python encounter specs, one per `.run` enemy string. Source enemy stats from Java decomp for HP/damage/hits.
- Cross-link: matches recommendation in `docs/research/engine-rs-audits/AUDIT_PARITY_STATUS.md:50` ("Acts 2-3: relic coverage ~54% overall, power coverage ~50% overall").

### 2.2 Deck reconstruction does not handle Pandora's Box / Astrolabe / Empty Cage transforms

- Symptom: `run_parser.py` forward-simulation has no handling for relics that transform deck contents at pickup. Pandora's Box (replace all Strikes+Defends with random cards from your color), Astrolabe (transform 3 cards), Empty Cage (purge 2 cards) all silently violate the simulator's deck state.
- Smoking gun: grep for `Pandora` in `packages/training/run_parser.py` returns nothing. The relic add path (`run_parser.py:322-327`) only special-cases PotionBelt for slot bumps.
- Impact: any Watcher run that picks up Pandora's Box, Astrolabe, or Empty Cage will have a desynced reconstructed deck from the point of pickup forward. `_validate_final_deck` catches it as a warning but combat replays in between are wrong.
- Fix scope (single-session, ~80 LOC): the `.run` schema records `cards_obtained` / `cards_removed` / `cards_transformed` per floor in event_choices for these relics. Wire those into the simulator: when relic == Pandora's Box, look at the `cards_obtained` and `cards_removed` for the same floor and apply.

### 2.3 Adaptation upgrade and other branching cards unaccounted

- Comment in run_parser does not list this as a TODO. Adaptation appears in the Watcher pool. If upgraded mid-combat as part of a card effect it does not go through `cards_obtained_event` — it just changes in-deck.
- Action: lower priority than §2.1/§2.2 because it does not affect entry deck for next combat in most cases.

### 2.4 Training observation contract not versioned end-to-end

- Issue surfaced in `docs/research/engine-rs-audits/INCONSISTENCY_REPORT.md:254-261`: "document and version the Rust ↔ Python observation contract".
- Today: `packages/engine-rs/src/training_contract.rs:21-28` declares `COMBAT_OBSERVATION_SCHEMA_VERSION = 1` but Python mirrors in `packages/training/contracts.py` are not version-checked at parse time. Schema drift will silently produce garbage.
- Fix scope (single-session, ~100 LOC): add explicit `schema_versions` validation in `bridge.py:parse_combat_puct_result` and `parse_combat_training_state`. Refuse to parse on mismatch with a clear error.

### 2.5 Legacy `packages/engine/` Python still pulled by `tests/` and `packages/parity/`

- `docs/architecture-overview.md:410-449` notes 90+ test files at top-level `tests/` and 4 files in `packages/parity/` import the legacy Python engine. The training stack does not depend on it.
- Issue: this is dead weight blocking a clean cut. Either migrate parity tooling to the Rust engine via PyO3 or accept a freeze and archive both.
- Fix scope (multi-session, ~500-1000 LOC of test rewrites or removals, depending on path).
- Recommendation: ship audit first, then propose freeze + archive in a separate PR.

### 2.6 71 cards on `complex_hook` legacy fallback (cross-link)

- Already documented in `docs/research/engine-rs-audits/COMPLEX_HOOK_AUDIT.md:25-30`. Watcher-relevant: Lesson Learned is the only one in the `5 truly complex` set that hits Watcher decks; rest are non-Watcher and don't impact this training stack.
- Action: no new work; tracked in the existing audit.

### 2.7 Solver "outperforms" human is currently unprovable

- Tonight: Jaw Worm 0.39 vs 6, Cultist 0.72 vs 0, Guardian 0 vs 9 (per the report). Some of this is genuine PUCT skill — Jaw Worm against a Watcher A0 deck is winnable with low HP loss given the deck contains JustLucky and Adaptation. But:
  - §1.1 means enemies always pick the same intent, so the solver is fighting deterministic dummies.
  - §1.2 means the "0.39" is a value-head estimate, not an integer HP outcome.
  - §1.3 means the solver gave up on convergence and reported a noisy snapshot.
- Until §1.1, §1.2, §1.3 land, no claim about solver-vs-human is meaningful. The replay's purpose is observability, not validation.

### 2.8 `RecordedCombatCase` does not store `entry_gold`

- `packages/training/run_parser.py:106-121` `RecordedCombatCase` has `entry_gold: int` field but `replay_recorded_run` (`run_replay.py:213-222`) never passes gold to `RustCombatEngine`. Gold matters only for combats that interact with relics like Old Coin or Bloody Idol mid-combat — minor but worth noting.

### 2.9 Action-target alignment: `RustCombatEngine` constructor is positional with 7 args

- `run_replay.py:213-222` calls `RustCombatEngine(case.entry_hp, case.max_hp, 3, list(case.entry_deck), spec.to_engine_enemies(), 7_000 + case.floor, list(case.entry_relics))`. Keyword-arg or struct-shaped constructor would catch the energy=3 bug at code-review time. PyO3 supports this.

## §3 Fix order / dependencies

Topological ordering. Each fix unlocks the next.

```
1.1 (enemy AI RNG)
   |
   +--> meaningful solver-vs-human delta
   +--> §5.1 test, §5.7 multi-enemy test
   |
   v
1.2 (per-turn replanning, integer HP)
   |
   +--> §5.2 integer HP regression test
   +--> turns "5 solved" into a verifiable claim
   |
   v
1.3 (iterative deepening, goal-conditioned search)
   |
   +--> §5.3 convergence test
   +--> "search until matches or beats human" feature
   |
   v
2.1 (Acts 2-4 encounter catalog)
   |
   +--> §5.5 catalog-completeness test
   +--> turns 18/23 unsupported into 0 unsupported
   +--> opens Acts 2-4 to validation pressure (which will surface 1.7-style fractional bugs in those enemies)

[parallel branch — does not block above]
1.4, 1.5, 1.7, 2.2 (entry state / deck reconstruction): single-session bug fixes; can be done any time but should land before claiming reconstruction is solid.

[parallel branch — also does not block]
2.4 (schema versioning), 2.5 (legacy archive plan): independent cleanup.

[deferred until after all of above]
1.6 (batched evaluator), 2.3 (in-combat upgrades), 2.8/2.9 (interface polish).
```

## §4 Estimated effort (rough)

| Item | Effort | Dependencies |
|---|---|---|
| 1.1 enemy AI RNG | multi-week (~20+ enemies × probabilistic logic, but simple per-enemy) | none |
| 1.2 per-turn replanning + integer HP | multi-session (~2-3 sessions) | 1.1 helpful but not required |
| 1.3 iterative deepening / goal-conditioned | single-session | 1.2 |
| 1.4 energy from relics | single-session (<1 hour) | none |
| 1.5 Neow 10% HP loss | single-session (<1 hour) | none |
| 1.6 batched evaluator | multi-week | 1.1, 1.2, 1.3 stable |
| 1.7 Neow transform/upgrade/three-cards | single-session | none |
| 2.1 encounter catalog Acts 2-4 | multi-session (~2 sessions; lots of mechanical lookup) | none |
| 2.2 Pandora's Box etc | single-session | none |
| 2.4 schema versioning | single-session | none |
| 2.5 legacy archive plan | multi-week (test migration) | independent |
| §5 test suite (8-15 tests) | multi-session (~2-3 sessions for the priority tests) | depends on the bug being tested |

## §5 New test suite scope

Naming: Rust tests live under `packages/engine-rs/src/tests/`. Python tests live under `tests/training/`.

### §5.1 `packages/engine-rs/src/tests/test_enemy_ai_rng_parity.rs`
- `test_jaw_worm_rollmove_consumes_rng`: build two `EnemyCombatState`s for JawWorm with the same move history but different RNG seeds; assert that with the §1.1 fix in place, the next-move distribution differs across seeds and matches Java's empirical distribution (sample 10k seeds, assert frequency within tolerance).
- `test_acid_slime_m_intent_distribution_matches_java`: same shape for AcidSlime_M.
- `test_first_turn_does_not_consume_rng`: assert `first_turn=true` enemies skip the `aiRng.random(99)` draw (Java parity).

### §5.2 `tests/training/test_recorded_run_integer_hp.py`
- `test_solver_hp_loss_is_integer_for_known_seed`: replay a known seed (e.g. tonight's `1776347657`) under fixed RNG, assert `solver_hp_loss` is an integer (or list of integers, one per stochastic replay) and falls within ±2 HP of recorded for at least 3 of the 5 currently-supported combats.
- Requires §1.1 + §1.2.

### §5.3 `packages/engine-rs/src/tests/test_iterative_deepening.rs`
- `test_iterative_deepening_reaches_converged_or_goal`: synthetic 1-enemy combat where convergence is achievable in 5000 visits; assert the deepening helper returns `Converged` or hits the goal predicate, never `TimeCap`.

### §5.4 `tests/training/test_entry_energy_relics.py`
- `test_energy_modifier_relics_change_entry_state`: build a recorded case with `Mark of the Bloom` in relics, assert `RustCombatEngine` constructor receives reduced energy. Today it always receives 3.

### §5.5 `tests/training/test_neow_reconstruction.py`
- `test_neow_ten_percent_hp_loss_drops_entry_hp`: parse a synthetic `.run` with `neow_cost=TEN_PERCENT_HP_LOSS`, assert floor-1 `RecordedCombatCase.entry_hp == int(0.9 * max_hp)`.
- `test_neow_remove_two_picks_strike_defend_default`: existing behavior, lock it.

### §5.6 `tests/training/test_neow_extended_bonuses.py`
- `test_neow_transform_upgrade_threecards_reconstruction`: synthetic runs covering `TRANSFORM_CARD`, `UPGRADE_CARD`, `THREE_CARDS` Neow bonuses; assert reconstructed deck matches `master_deck` after each.

### §5.7 `packages/engine-rs/src/tests/test_multi_enemy_intent_order.rs`
- `test_three_sentries_intent_order_matches_java`: build a 3-Sentry combat, step 5 turns under fixed seed; assert the per-turn intent sequence per Sentry matches a Java-recorded golden (capture once, freeze).
- `test_centurion_healer_pair_order_matches_java`: same for Centurion+Healer.
- `test_lots_of_slimes_intent_order_matches_java`: same for Acts-2 Lots of Slimes.

### §5.8 `tests/training/test_run_parser_deck_reconstruction.py`
- `test_reconstructed_deck_equals_master_deck_for_tonights_seed`: parse `runs/WATCHER/1776347657.run`, assert `len(run.reconstruction_warnings) == 0` and `sorted(reconstructed_deck) == sorted(run.final_master_deck)`. Currently warnings are logged but not fatal — flip to assertion in test.

### §5.9 `tests/training/test_pandoras_box_reconstruction.py`
- `test_pandoras_box_replaces_strikes_and_defends`: synthetic `.run` with PandorasBox pickup; assert the simulator reads the `cards_obtained` for that floor and the resulting deck matches.

### §5.10 `tests/training/test_encounter_catalog_completeness.py`
- `test_python_catalog_covers_all_recorded_encounters_in_corpus`: scan `runs/WATCHER/*.run` for distinct `damage_taken[].enemies` strings, assert every one resolves via `encounter_spec`. Today: 18 encounters fail.
- `test_python_catalog_covers_rust_known_enemy_ids`: assert every Rust `known_enemy_ids()` entry has a Python encounter that uses it.

### §5.11 `packages/engine-rs/src/tests/test_combat_solver_integer_hp.rs`
- `test_terminal_outcome_is_integer_after_replay`: stand up a CombatSolver, run PUCT, then play out the chosen action sequence step-by-step until terminal, assert final HP delta is integer and matches `terminal_outcome().expected_hp_loss` only when terminal was actually reached. Locks §1.2 fix.

### §5.12 `tests/training/test_recorded_run_replay_golden.py`
- `test_tonights_seed_produces_expected_replay_summary`: end-to-end. Run `replay_recorded_run` against the snapshotted `.run`, assert `(solved, failed, unsupported, error) == (5, 0, 18, 0)` today and update the golden as fixes land. Catches drift in either direction.

### §5.13 `packages/engine-rs/src/tests/test_search_stop_reason_distribution.rs`
- `test_hallway_combats_converge_under_increased_budget`: build a small set of hallway combats, run search at hallway preset; assert at least 70% reach `Converged` (today: 0%).

### §5.14 `tests/training/test_observation_schema_version.py`
- `test_bridge_rejects_old_schema_version`: feed a synthetic V0 (or V2 unknown) payload, assert `parse_combat_puct_result` raises with a clear error. Locks §2.4.

### §5.15 `packages/engine-rs/src/tests/test_enemy_first_turn_intent_parity.rs`
- `test_first_turn_intents_match_java_for_all_act1_enemies`: golden table `enemy_id -> first-turn move_id`, assert match for all `known_enemy_ids()` Act 1 entries (catches the simple "did the registry change a damage value" regressions).

## §6 Recommended next sprint

If only one sprint: §1.1 (enemy AI RNG, the only structural bug — everything else is patch-shaped) → §1.2 (per-turn replanning so HP losses are integers) → §5.1, §5.2, §5.7, §5.11 (the four tests that lock those fixes and prevent regression) → §2.1 (Acts 2-4 encounter catalog so the next replay run can actually exercise the engine surface that was previously dark). Land §1.3 (iterative deepening) only after §1.1+§1.2 because it changes search semantics and the test golden in §5.12 depends on stable search behavior. Defer §1.6 (batched evaluator), §2.5 (legacy archive), and §2.4 (schema versioning) to follow-on PRs — they are cleanup, not parity.
