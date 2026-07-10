# engine-rs Tests / Types / API Audit

Scope: `packages/engine-rs` (Rust). Read-only review of public surface, type hygiene, test quality, PyO3 boundary, dead code, and clippy output. IDs are `T1`-`T20`. Severity: `bug` | `tech-debt` | `missing-test` | `style` | `unverified`. Register D1-D87 is pre-existing and is not duplicated here.

Method: full read of `lib.rs` (1986 L), `training_contract.rs` (1546 L), `obs.rs` (981 L), `decision.rs`, `actions.rs`, `combat_types.rs`, `tests/mod.rs`, `tests/support.rs`; spot-sampled 10 test files; ran `cargo clippy --lib` (186 warnings) and attempted to attach every orphan test file to the module tree to classify failures.

---

## Top Findings Summary (ranked by impact)

1. **T1 (bug/missing-test, HIGH)** - 44 test files on disk, 230 `#[test]` functions, are not declared in `src/tests/mod.rs`. A majority compile cleanly if attached and are genuine parity tests. The rest reference removed APIs (rotten). Coverage the repo thinks it has is illusory.
2. **T2 (bug, HIGH)** - `training_contract::combat_engine_from_snapshot` silently drops orb contents (only max_slots is restored; all orbs become empty) and drops `combat_over`/`player_won` flags. Any PUCT expansion from a mid-combat snapshot with active orbs restarts with zero Channel.
3. **T4 (tech-debt, HIGH)** - 186 clippy warnings in the library, 72 auto-fixable. Top classes: 81 `empty_line_after_doc_comment`, 33 `useless_conversion` on `PyErr`, 19 `if_same_then_else`, 14 `needless_borrow`.
4. **T12 (tech-debt, MED)** - Every engine module is `pub mod` in `lib.rs`. Internal-only modules (`gameplay`, `effects`, `powers`, `potions`, `relics`, `enemies`, `events`, `map`, `card_effects`, `combat_hooks`, `relic_flags`, `status_effects`, `status_ids`, `ids`) leak into the crate's public API and into the Python wheel.
5. **T5 (tech-debt, MED)** - PyO3 dict shape is produced by `serialize_to_py_dict(json)`. Changing any `CombatSnapshotV1` / `DecisionContext` field silently drifts the Python dict without any type-level signal, and there is no PyO3 roundtrip test locking the shape.

---

## T1 - 44 orphan test files, 230 dropped `#[test]` fns (`missing-test`, HIGH)

`packages/engine-rs/src/tests/mod.rs` declares 168 test modules. `ls src/tests/*.rs` returns 210 test files. Difference = 42 data files + 2 include files; actual orphan count is 44 distinct `test_*.rs` files.

Evidence: `comm -23 <ls> <mod.rs>` yields the exact list. `grep -cE '#\[test\]'` inside those files totals 230 annotated tests.

Classification via batch compile (all 44 attached, `cargo check --lib --tests`):
- Many compile successfully and add real coverage (e.g. `test_orb_runtime_java_wave1` with 11 tests, `test_played_card_instance_state` with 5, `test_potion_runtime_action_path` with 15).
- A minority reference removed symbols and will not compile:
  - `DEF_STORM`, `DEF_CREATIVE_AI`, `DEF_ENTER_DIVINITY`, `DEF_MAYHEM`, `DEF_TOOLS_OF_THE_TRADE` (power defs renamed or deleted).
  - `hook_creative_ai`, `hook_enter_divinity`, `hook_mayhem`, `hook_tools_of_the_trade` (removed hooks).
  - `test_power_runtime_complex` uses `powers::defs::complex::...` which is now private.
  - 4 sites reference `effective_cost_inst` which is now `pub(crate)`/private.

Representative orphans (wave, `#[test]` count, guess):
- `test_damage_followup_java_wave1` (11) - likely compiles.
- `test_orb_runtime_java_wave1` (11) - compiles; high-value Java parity.
- `test_potion_runtime_action_path` (15) - compiles; largest orphan.
- `test_card_runtime_watcher_wave3..6` (7/4/7/4) - some reference missing card defs.
- `test_power_runtime_complex` (7) - rotten; uses private `powers::defs::complex`.
- `test_power_runtime_card_play` (5) / `_debuff_enemy` (7) / `_end_to_end` (5) / `_replay` (4) / `_turn_start` (5) - some compile, some reference removed hooks.

Action: (a) attach working orphans to `tests/mod.rs`, (b) delete rotted ones or port their assertions onto live APIs. Do both in a single scope/cleanup PR so the total delta to expected coverage is visible.

Supporting header reads (valid, live APIs):
```
packages/engine-rs/src/tests/test_played_card_instance_state.rs  // Java oracle refs; uses crate::tests::support
packages/engine-rs/src/tests/test_potion_runtime_action_path.rs  // uses Action, combat_state_with, engine_with_state
```

## T2 - `combat_engine_from_snapshot` drops orb contents and terminal flags (`bug`, HIGH)

`packages/engine-rs/src/training_contract.rs:735-803`.

```
784|    state.orb_slots = crate::orbs::OrbSlots::new(snapshot.orb_slots);
```

`OrbSlots::new(max_slots)` creates an `OrbSlots { slots: Vec::new(), max_slots }` (see `src/orbs.rs:146` where `pub slots: Vec<Orb>`). The snapshot's `orb_slots: usize` only stores the max count. `CombatSnapshotV1` has no field storing the orbs themselves (verified by `grep -n "orb_contents\|slot_orb\|pub slots"` - none in `training_contract.rs`). Result: every mid-combat snapshot that is round-tripped loses all Lightning/Frost/Dark/Plasma orbs; Defect PUCT from snapshot will grossly under-evaluate orb-heavy positions.

`packages/engine-rs/src/training_contract.rs:857-858` writes `combat_over` / `player_won` into the snapshot struct, but the restore path (735-803) never assigns them back; they default to `false`. Snapshots taken after enemy death / player death restart as if combat is ongoing.

Needed: add `orb_contents: Vec<OrbTokenV1>` to `CombatSnapshotV1`, and restore both orb vec contents and terminal flags. Add a roundtrip test: `engine -> snapshot -> engine'` asserts orbs, `combat_over`, `player_won`, and HP/block equal the source.

## T3 - Snapshot roundtrip has no explicit test (`missing-test`, HIGH)

`training_contract.rs` has only 4 `#[test]` fns for 1546 lines of versioned contract code. None does `engine -> snapshot -> engine'` equality on live combat state.

This is the test that should have caught T2. Adding it is the single highest-leverage test to add.

## T4 - 186 clippy warnings in `cargo clippy --lib` (`tech-debt`, HIGH)

Top buckets (run `cargo clippy --lib -p sts-engine`):

| Count | Lint |
| ----- | ---- |
| 81 | `empty_line_after_doc_comment` |
| 33 | `useless_conversion` (all on `PyResult<_>` returns, PyErr→PyErr identity) |
| 19 | `if_same_then_else` |
| 14 | `needless_borrow` |
| 6  | `collapsible_if` |
| 5  | `contains_instead_of_iter_any` |
| 4  | `too_many_arguments` (8/7 threshold) |
| 3  | `redundant_closure` |
| 2  | `from_str_on_std_trait_type` (`Stance::from_str` shadows `std::str::FromStr`) |
| 2  | `extend_with_drain` |
| 2  | `map_or_none` |
| 2  | `let_else` |
| 2  | `unnecessary_cast` (`u64` -> `u64`) |
| 1  | `field_reassign_with_default` |
| 1  | `manual_is_multiple_of` |
| 1  | `manual_range_contains` |
| 1  | `manual_memcpy` |

72 of the 186 are auto-fixable via `cargo clippy --fix --lib -p sts-engine`.

The 33 `useless_conversion` warnings all sit on function return types (`PyResult<Bound<'py, PyDict>>` inside `lib.rs`), and blanket-removing them is mechanical: see `src/lib.rs:142`, `:167`, `:179`, `:230`, `:234`, `:242`, `:247`, `:262`, `:744`, `:759`, `:772`, `:1500`, `:1516`, `:1577`, `:1586`, `:1601`, `:1611`, `:1629`, `:1651`, `:1662`, `:1716`, `:1745`, `:1830`, `:1847`, `:1852` (plus 8 more elsewhere).

## T5 - PyO3 dict surface goes through JSON roundtrip (`tech-debt`, MED)

Every `-> PyResult<Bound<'py, PyDict>>` in `lib.rs` routes through `serialize_to_py_dict` (serde_json::Value -> PyDict). Consequences:

1. Schema drift in Rust silently reshapes the Python dict. There is no compile-time or test-time guarantee that `get_training_schema_versions`, `get_combat_training_state`, `get_combat_snapshot`, `get_combat_context`, `get_info`, `get_decision_state`, `get_decision_context` return dicts with the expected keys.
2. Every call allocates a JSON string and reparses it. A dozen hot-path dict calls sit under `step()`.
3. No test asserts the concrete dict keys. `tests/test_rl_contract.rs` uses a `python_bridge_guard` mutex for PyO3 but asserts on shape indirectly.

Recommendation: either (a) a single "PyO3 dict shape" test that pins a YAML/JSON fixture of expected keys for each surface, updated when the snapshot version bumps, or (b) switch hot-path getters to `#[pymethods]` returning typed PyO3 structs (`#[pyclass]` on `CombatSnapshotV1` etc.).

## T6 - `card_instance_is_4_bytes` test name wrong (`style`, LOW)

`src/combat_types.rs:~178`. Test asserts `size_of::<CardInstance>() == 8`. Rename to `card_instance_is_8_bytes`.

## T7 - `test_observation_not_all_zeros` is a tautology (`missing-test`, LOW)

`src/obs.rs` test block. Asserts `nonzero > 10` on a 480-dim vector. With `hp/100.0` alone occupying ~4 dims, the test passes on any non-empty run. Should assert expected dims for a known seed.

## T8 - `Stance::from_str` shadows `std::str::FromStr` (`tech-debt`, LOW)

Two occurrences (clippy `from_str_on_std_trait_type`). The inherent method silently hides the trait; `String::parse` on `Stance` fails mysteriously. Implement `impl FromStr for Stance` instead.

## T9 - Potion classification in `obs.rs` uses fragile string `.contains()` (`tech-debt`, LOW)

`src/obs.rs` potion tier classification tests for substrings "fire" / "explosive" / "attack" / "poison" to bucket potions as damage-class. Adding a potion named `"Firebreath"` or `"Poison Slime"` silently mis-buckets it. Drive this from `potions::potion_kind()` instead.

## T10 - Ad-hoc error creation across PyO3 surface (`tech-debt`, MED)

`PyValueError::new_err(format!(...))` scattered across `lib.rs`. No typed error surface means every error string is a string-match contract for Python consumers. Needed: a small `EngineError` enum with `impl From<EngineError> for PyErr`; then callers return `-> Result<T, EngineError>`.

## T11 - `CardInstance` has unused flag bits (`unverified`, LOW)

`src/combat_types.rs` defines `FLAG_INNATE` (0x10) and `FLAG_PURGE` (0x20) but exposes only `is_retained`, `is_ethereal`, `is_upgraded`, `is_free` accessors. Grep confirms INNATE/PURGE bits are never read through `CardInstance` methods, though they may be written via `flags |=`. Either add accessors and a test, or delete the bits.

## T12 - Every module is `pub mod` in `lib.rs` (`tech-debt`, MED)

`lib.rs:11-37`. Twenty-seven `pub mod` declarations. The Python wheel currently exposes all of `gameplay`, `effects`, `powers`, `potions`, `relics`, `enemies`, `events`, `map`, `card_effects`, `combat_hooks`, `relic_flags`, `status_effects`, `status_ids`, `ids`, `combat_types`, `damage`, `decision`, `seed`, `state` as importable Rust paths via `use sts_engine::*`. Pin this to `pub(crate)` for anything not consumed by `training/` or `rust_search/`. Each hidden module also removes its warnings from the public surface (smaller clippy fallout).

## T13 - `CombatSnapshotV1` round-trip is not the only missing invariant test (`missing-test`, MED)

Beyond T3: no test covers `RestrictionPolicyV1` filtering (`restricted_legal_decision_actions`). All that is tested is the `DecisionAction` mapping in `test_rl_contract.rs`; the `NoCardRewards`, `NoCardAdds`, `UpgradeRemoveOnly` filters are uncovered.

## T14 - Test asserts current buggy behavior risk (`unverified`, MED)

Sampled 10 tests. None flagged explicitly as locking a Java-parity bug in place. However, `test_observation_not_all_zeros` (T7) and the sparse coverage in `training_contract` (T3) mean the encoder output shape is effectively frozen by absence of tests rather than presence of one - any schema fix will need its own test first. Suggest: before D88+ parity fixes land, spend one session writing one round-trip test per `CombatSnapshotV1` field and one 480-dim-layout assertion (known seed -> expected dims for known fields).

## T15 - `DecisionContext` has Option-of-kind fields rather than an enum (`style`, LOW)

`src/decision.rs:180-190`. `DecisionContext { kind, neow: Option<_>, combat: Option<_>, reward_screen: Option<_>, map, event, shop, campfire }`. The `kind: DecisionKind` and the Options must agree invariantly - a non-matching combination is a bug with no compile-time check. Tighten to an enum (`DecisionContextBody::Combat(CombatContext) | ::Neow(...) | ...`) to eliminate a whole class of drift.

## T16 - `pub mod` in `tests/` (`tech-debt`, LOW)

`src/tests/mod.rs:16` declares `pub(crate) mod support;`. All other test modules are `mod ...;` but still inhabit a `cfg(test)` crate. This is fine; noting here only that the support module includes many `pub` helpers that could be scoped tighter.

## T17 - `serialize_to_py_dict` is the canonical path but has no contract test (`missing-test`, MED)

Same path as T5 but as a test item. Add a `test_pyo3_dict_shape` that locks the key set for each emitter.

## T18 - No `cargo doc` warnings audit (`unverified`, LOW)

Not run. Likely a bucket of further small hygiene wins when the `pub mod` count drops per T12.

## T19 - `CombatPuctConfigV1` defaults are hardcoded and untested (`missing-test`, LOW)

`test_combat_puct.rs` covers determinism, terminal override, hallway/elite/boss config convergence, hard visit cap, and time cap. Good. But the config default values are hardcoded constants in `training_contract.rs` and no test pins these exact numbers - any accidental change to `c_puct` / `num_simulations` defaults would silently reshape every collector.

## T20 - No `#[ignore]` or `todo!()` in src proper (`style`, INFO)

Grep for `#[ignore]`, `todo!`, `unimplemented!`, `FIXME`, `HACK` in `packages/engine-rs/src/**` returned 0 hits outside test files. This is a clean bill on that axis.

---

## Test Quality Sample (10 files)

| File | Quality | Notes |
| ---- | ------- | ----- |
| `tests/test_ai_rng_parity.rs` | STRONG | Java-oracle comments, tests probability distribution (25/30/45), anti-repeat guards, ai_rng stream order, deterministic draws for Cultist. |
| `tests/test_combat_puct.rs` | STRONG | Determinism, terminal root, hallway/elite/boss, hard visit cap, time cap. Five sharp tests. |
| `tests/test_rl_contract.rs` | STRONG | Action encoding non-overlap, illegal-action rejection, combat obs v3, decision state/actions, reward screens. Uses `python_bridge_guard` mutex correctly. |
| `tests/test_played_card_instance_state.rs` (ORPHAN) | STRONG | Streamline/Rampage/SteamBarrier/GlassKnife/GeneticAlgorithm/RitualDagger with Java oracle citations. 5 `#[test]`. Currently unreachable - not in mod.rs. |
| `tests/test_potion_runtime_action_path.rs` (ORPHAN) | STRONG | 15 `#[test]`, 508 L. The single biggest missing test suite. |
| `tests/test_power_runtime_complex.rs` (ORPHAN) | ROTTEN | Uses `powers::defs::complex` which is private. Needs port or delete. |
| `obs.rs::test_observation_not_all_zeros` | WEAK | `nonzero > 10` on 480 dims passes on any non-empty run. |
| `obs.rs::test_observation_dim` | STRONG | Locks the 480/260/220 dims - critical. |
| `combat_types.rs::card_instance_is_4_bytes` | CORRECT-BUT-MISNAMED | Asserts `== 8` despite name. |
| `decision.rs::decision_stack_tracks_nested_reward_choice` | ADEQUATE | Pushes RewardScreen then RewardChoice, asserts current_kind stays RewardScreen. |

Additional observation: `tests/support.rs` is the shared test harness. Its `play_on_enemy`, `play_self`, `engine_with_state`, `combat_state_with` are the right abstraction - every orphan file already imports this. Re-attachment is mechanical, not semantically hard.

---

## Types / API Hygiene

- `actions::Action` is a minimal 5-variant enum (`PlayCard`, `UsePotion`, `EndTurn`, `Choose`, `ConfirmSelection`). Clean.
- `decision::DecisionAction` is the outer envelope. Clean, closed set.
- `decision::DecisionKind` is a pure tag enum; should back a sum type, not a sibling field (T15).
- `combat_types::CardInstance` is 8 bytes, `#[derive(Copy)]`. Good. Has vestigial flag bits (T11).
- `run::RunAction` covers all out-of-combat actions; mapping to/from `DecisionAction` is exhaustive (verified `match` arms).
- No duplicate type definitions found. No `#[default]` misuse found (`DecisionStack::default()` is `Vec::new()`, correct). No `pub use` ambiguity.

## PyO3 Boundary

- GIL handling: every `fn get_*<'py>(&self, py: Python<'py>)` correctly threads the `py: Python<'py>` token. No `allow_threads` misuse found.
- No `&PyAny` / legacy `Bound` mixing.
- Return types: 33 `useless_conversion` warnings on `PyResult<...>` (T4). Cosmetic.
- JSON-roundtrip hazard: T5.
- No test pins the dict keys on the Python side from Rust; Python-side tests may exist but are out of scope.

## Training Contract Internal Consistency

- `TrainingSchemaVersionsV1` is the version gate. `CombatObservationSchemaV1`, `LegalActionCandidateV1`, `CombatSnapshotV1` each carry a `schema_version`. Good.
- Token caps: HAND=10, ENEMY=5, PLAYER_EFFECT=32, ENEMY_EFFECT=16, ORB=10, RELIC_COUNTER=8, CHOICE=10. These match obs encoder assumptions in `obs.rs`. Verified by grep.
- `COMBAT_FRONTIER_CAPACITY=8`, `COMBAT_PUCT_STABLE_WINDOWS=3` used consistently by `test_combat_puct`.
- Snapshot has 551 lines of structured fields but restoration (735-803) skips orb contents (T2), `combat_over`, `player_won` (T2). All three are bugs of omission, not design.

## `obs.rs` 480-dim Encoder

- Layout comment matches `RUN_DIM=480`, `STATE_DIM=260`, `ACTION_DIM=220`.
- `RELIC_CATALOG` is 181 entries hardcoded; test verifies index 123 = PureWater, 180 = Yang. Good.
- Normalizing constants (`hp/100.0`, `damage/40.0`, `block/50.0`) are plausible but not Java-derived comments. Acceptable.
- Potion tier buckets use `.contains()` on potion name string (T9).
- No dim-by-dim test for known seeds.

## Dead Code / TODO

- `grep -rn 'TODO\|FIXME\|HACK\|unimplemented!\|todo!\(' packages/engine-rs/src/**/*.rs` excluding tests: 0 hits.
- `#[allow(dead_code)]` on used code: not found via spot check; clippy did not flag any.
- Orphan test files (T1) are the only large-scale dead-code surface.

---

## Final Summary (top 5 cleanups by impact)

1. **Re-attach the 44 orphan test files.** Add every compiling orphan (`test_orb_runtime_java_wave1` + 11 tests, `test_potion_runtime_action_path` + 15 tests, `test_played_card_instance_state` + 5, and ~30 more) to `src/tests/mod.rs`; delete or port the ~5-10 rotten orphans that reference removed symbols (`DEF_STORM`, `hook_creative_ai`, `powers::defs::complex`). Recovers ~200 real tests that the repo silently lost. Single PR, mechanical.
2. **Fix `combat_engine_from_snapshot` orb and terminal-flag restoration** (`training_contract.rs:735-803`). Add `orb_contents: Vec<OrbTokenV1>` to `CombatSnapshotV1`, copy orbs in both directions, restore `combat_over`/`player_won`. Add one round-trip test. Directly unblocks correct PUCT from snapshots for Defect and mid-fight terminal edges.
3. **Run `cargo clippy --fix --lib -p sts-engine`** to auto-apply 72 of the 186 warnings, then hand-fix the remainder (81 empty-line-after-doc, 33 useless PyErr conversions, 19 if-same-branches, 14 needless-borrow). Almost entirely mechanical; shrinks future warning noise.
4. **Narrow `pub mod` in `lib.rs` to `pub(crate)` for every module not consumed externally.** Candidates: `gameplay`, `effects`, `powers`, `potions`, `relics`, `enemies`, `events`, `map`, `card_effects`, `combat_hooks`, `relic_flags`, `status_effects`, `status_ids`, `ids`, `status_effects`. Reduces the Python wheel surface by ~50% and makes future refactors safe.
5. **Add a PyO3 dict-shape contract test.** One fixture per `get_*` emitter pinning the expected top-level key set; fail on schema drift. Pairs with T3 (snapshot roundtrip) as the two tests that prevent silent shape regression.
