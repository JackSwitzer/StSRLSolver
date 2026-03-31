# Dead Code Audit -- 2026-03-31

Scanned `packages/training/` (23 files, ~12k LOC), `packages/engine/` (55 files, ~59k LOC),
`packages/engine-rs/` (11 Rust files, ~7.9k LOC on `feat/rust-engine-expansion`),
and `scripts/` (5 Python files) for unused imports, dead functions, unreachable code,
stale config keys, and duplicate utilities.

## Summary

| Category | Found | Removed | Deferred |
|----------|-------|---------|----------|
| Unused imports (training) | 39 | 18 | 21 (`__future__.annotations` -- harmless) |
| Unused imports (engine) | 206 | 44 | 162 (`__init__.py` re-exports = public API) |
| Dead functions/methods | 3 | 3 | 0 |
| Stale config keys | 4 | 4 | 0 |
| Registry overwrites (bugs) | 2 | 0 | 2 (see below) |
| Duplicate top-level functions | 5 | 0 | 5 (cross-module, need investigation) |
| Rust dead code | 1 | 0 | 1 (trivial, 3-line getter) |
| Stale scripts | 0 | 0 | 0 (all wired into training.sh) |

**Net: 44 lines removed, 38 files cleaned. 0 test regressions (6264 pass).**

## Removed -- Unused Imports (training/)

| File | Removed |
|------|---------|
| strategic_trainer.py | `math`, `field`, `_get_device` |
| iql_trainer.py | `_get_device` |
| grpo_trainer.py | `field`, `Any`, `_get_device` |
| mlx_inference.py | `Dict`, `Optional` |
| reward_config.py | `Any` |
| offline_data.py | `field`, `Optional`, `Tuple` |
| conquerer.py | `field` |
| shared_inference.py | `Tuple` |
| turn_solver.py | `field` |
| state_encoders.py | `Any`, `Tuple` |
| gym_env.py | `GamePhase` |

## Removed -- Unused Imports (engine/)

| File | Removed |
|------|---------|
| combat_engine.py | `Callable` |
| content/powers.py | `Callable`, `math` |
| content/enemies.py | `Callable`, `math` |
| content/events.py | `Callable` |
| content/cards.py | `Callable` |
| content/stances.py | `Optional` |
| utils/java_hashmap.py | `Dict`, `field` |
| utils/card_library_order.py | `Dict`, `sys` |
| state/rng.py | `List` |
| state/game_rng.py | `Optional` |
| effects/defect_cards.py | `Optional`, `Dict`, `Any`, `effect_custom`, `trigger_orb_passives` |
| effects/registry.py | `Union` |
| effects/cards.py | `effect_custom` |
| effects/orbs.py | `field` |
| effects/executor.py | `get_effect_handler`, `EffectTiming`, `list_registered_effects` |
| handlers/rooms.py | `TYPE_CHECKING`, `CardInstance`, `GameRNG` |
| handlers/event_handler.py | `deepcopy` |
| handlers/combat.py | `Tuple`, `GameRNG`, `calculate_block`, `calculate_incoming_damage`, `WRATH_MULT`, `DIVINITY_MULT`, `CardType`, `Intent`, `MoveInfo`, `EnemyType`, `RelicContext`, `trigger_orb_start_of_turn` |
| handlers/shop_handler.py | `Dict`, `Tuple`, `Any`, `CardType`, `CardColor`, `ALL_RELICS`, `ALL_POTIONS` |
| generation/relics.py | `Dict`, `deepcopy` |
| generation/map.py | `py_random` |
| generation/rewards.py | `Any`, `Enum` |
| generation/shop.py | `field`, `Any`, `Enum` |
| registry/relic_factories.py | `Any`, `wraps` |

## Removed -- Dead Functions/Methods

| Location | Name | Reason |
|----------|------|--------|
| `offline_data.py:78` | `OfflineDataset.to_torch()` | Never called; `sample_batch()` is used instead |
| `seed_pool.py:110` | `SeedPool.unique_seeds` | Property defined but never read |
| `training_runner.py:182` | `OvernightRunner.get_current_sweep_config()` | Defined but never called anywhere |

## Removed -- Stale Config Keys

| Key | File | Reason |
|-----|------|--------|
| `ENTROPY_FLOOR_AVG_FLOOR` | training_config.py | Referenced nowhere outside config |
| `LR_WARMUP_STEPS` | training_config.py | Referenced nowhere outside config |
| `LR_COMBAT_NET` | training_config.py | Referenced nowhere outside config |
| `PBRS_WEIGHTS` | training_config.py | Referenced nowhere; `compute_potential()` hardcodes its own weights |

## BUGS FOUND -- Registry Overwrites (not removed, needs separate fix)

Two decorator-registered functions are defined twice under the same key.
The second definition silently overwrites the first, making it unreachable.

| Function | File | Lines | Impact |
|----------|------|-------|--------|
| `damage_all_x_times` | effects/cards.py | L2148 (Whirlwind) vs L2791 (Dagger Spray) | Whirlwind handler is dead -- the Dagger Spray version wins |
| `plated_armor_hp_lost` | registry/powers.py | L897 vs L1994 | First definition is dead -- second version overwrites it |

These should be fixed by giving each a unique registry key.

## Duplicate Top-Level Functions (not removed, needs investigation)

These functions share the same name across different modules. Some may be intentional
wrappers; others may be redundant copies.

| Function | Locations | Notes |
|----------|-----------|-------|
| `create_enemy` | `content/enemies.py:6266`, `state/combat.py:575` | Different signatures -- enemies.py creates AI-enabled enemy, combat.py creates combat state entity |
| `generate_shop_inventory` | `handlers/shop_handler.py:175`, `generation/rewards.py:972` | Likely legacy duplication -- shop_handler version may delegate |
| `get_potion_by_id` | `content/potions.py:717`, `generation/potions.py:520` | Same purpose, two locations |

## Rust Engine Audit (feat/rust-engine-expansion)

Scanned 11 files, ~7.9k LOC. The Rust code is clean.

| Finding | Details |
|---------|---------|
| Dead function | `EnemyCombatState::is_attacking()` in `state.rs:173` -- defined but never called from Rust or PyO3 |
| All other `pub fn` | Used either from `engine.rs`, PyO3 wrappers, or `tests.rs` |
| No unused `use` statements | Clean |
| No `#[allow(dead_code)]` | None present |

## Scripts Audit

All 5 Python scripts in `scripts/` are actively wired:

| Script | Reference |
|--------|-----------|
| `pretrain_bc.py` | `training.sh` line 421, 426 |
| `pretrain_combat.py` | `training.sh` line 422, 427 |
| `pretrain_eval.py` | `training.sh` line 423, 428 |
| `push_metrics.py` | `training.sh` line 442 |
| `v3_experiment_sweep.py` | Standalone experiment runner, referenced in docs |

No scripts archived -- all in active use.

## Training Modes: IQL and GRPO

Both `iql_trainer.py` and `grpo_trainer.py` ARE wired into the main training loop:
- `training_runner.py:755` imports and instantiates `IQLTrainer` when `ALGORITHM == "iql"`
- `training_runner.py:765` imports and instantiates `GRPOTrainer` when `ALGORITHM == "grpo"`
- Both have test coverage in `test_training_v3.py`
- `v3_experiment_sweep.py` also imports `IQLTrainer`

These are not dead code -- they're alternative training algorithms selectable via config.

## Deferred -- Not Removed

### `__init__.py` re-exports (engine/)

`packages/engine/__init__.py` imports ~130 names from submodules and re-exports them.
These constitute the **public API**. Not dead code.

### Engine registry functions (powers, relics, effects)

~500 functions registered via decorators (`@relic_trigger()`, `@power_trigger()`,
`@effect()`). Called at runtime via dispatch. **False positives.**

### Engine/Training classes used within defining file only

`CombatLog`, `CombatLogEntry`, `PendingSelectionContext`, `RewardType`,
`SkipBossRelicAction`, `TriggerRegistry`, `HashMapEntry`, `MLXResidualBlock`,
`QNetwork`, `ValueNetwork`, `StrategicWeightSync`, `TorchStrategicBackend`,
`SearchNode` -- all used within their defining modules.

### `__future__.annotations` imports

21 files import `from __future__ import annotations`. Harmless, kept for forward compat.
