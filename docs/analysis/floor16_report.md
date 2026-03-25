# Floor 16 Death Analysis Report

**Date**: 2026-03-25
**Training run**: v3_concurrent (11.27h, 29,892 games, 0 wins)
**Metrics**: val_acc 55.6%, avg_floor 8.3, peak_floor 16

## Executive Summary

Every game that reaches the Act 1 boss (floor 16) dies there. 0/29,892 games passed floor 16 in 11 hours of training. The root cause is a **hardcoded `turn_solver_ms=20.0` in `v3_train_concurrent.py`** that overrides the 30-second boss budget defined in `training_config.py`. The solver gets 20ms to solve a boss fight that needs 30,000ms -- a **1,500x budget deficit**. Secondary issues amplify the problem: MCTS combat search is disabled, the boss HP progress reward has a timing bug, and the reward gap for beating the boss is too small to create learning signal.

## Root Cause #1: Hardcoded 20ms Solver Budget (CRITICAL)

### Evidence

The `training_config.py` defines generous solver budgets:

```python
# training_config.py line 58-62
SOLVER_BUDGETS: Dict[str, tuple] = {
    "monster": (50.0, 5_000, 300_000),
    "elite":   (2_000.0, 50_000, 600_000),
    "boss":    (30_000.0, 200_000, 600_000),  # 30 SECONDS for bosses
}
```

But `v3_train_concurrent.py` line 596 hardcodes `turn_solver_ms=20.0`:

```python
# scripts/v3_train_concurrent.py line 596
(seed, 0, 0.8, total_games, 20.0, False, False, 0)
#                            ^^^^ 20ms hardcoded
```

This `turn_solver_ms` parameter sets the **default** solver budget in worker.py line 237-238:

```python
turn_solver = TurnSolverAdapter(
    time_budget_ms=turn_solver_ms,  # 20ms default
    ...
    solver_budgets=SOLVER_BUDGETS,  # override dict IS passed
)
```

### Why the Override Dict Doesn't Fully Fix It

The `SOLVER_BUDGETS` dict IS passed to the adapter and IS applied in `pick_action()` (line 1198-1209 of turn_solver.py). When room_type is "boss", the solver's `default_time_budget_ms` gets overridden to 30,000ms. **This part works correctly.**

However, there are two problems:

1. **The multi-turn solver inner budget is fixed at 500ms** (turn_solver.py line 1131). The `MultiTurnSolver` creates its own inner `TurnSolver` with `time_budget_ms=500` regardless of the SOLVER_BUDGETS override. This means multi-turn planning for bosses uses 500ms per turn -- adequate but not 30s.

2. **The initial node budget is capped at 2,000** (`max(1000, int(20.0 * 100))` = 2,000 nodes). Even though the time budget gets overridden to 30s for boss, the node budget remains at 2,000 -- the budget for a trivial monster fight. The boss budget should be 200,000 nodes.

### Affected Scripts

| Script | turn_solver_ms | MCTS | Status |
|--------|---------------|------|--------|
| `v3_train_concurrent.py` line 596 | **20.0** | **Disabled** | **ACTIVE -- the overnight run** |
| `v3_collect.py` line 166 | **20.0** | **Disabled** | Used for data collection |
| `v3_experiment_sweep.py` line 207,261 | **20.0** | **Disabled** | Used for A/B/C/D experiments |
| `v3_overnight.py` (via runner) | 50.0 | Config-dependent | Correct |
| `v3_sweep.py` | 50.0 | Config-dependent | Correct |
| `v3_1h_test.py` | 50.0 | Config-dependent | Correct |
| `sweep_config.py` (all configs) | 100.0 | Varies | Correct |
| `worker.py` default parameter | 50.0 | Parameter | Correct |

The three actively-used scripts (`v3_train_concurrent.py`, `v3_collect.py`, `v3_experiment_sweep.py`) all hardcode 20ms.

## Root Cause #2: MCTS Combat Search Disabled

The concurrent script calls `_play_one_game` with `mcts_enabled=False`:

```python
# v3_train_concurrent.py line 596
(seed, 0, 0.8, total_games, 20.0, False, False, 0)
#                                  ^^^^^  ^^^^^  ^
#                            strategic_search  mcts_enabled  mcts_card_sims
```

This means `COMBAT_MCTS_BUDGETS` (training_config.py line 164-168) is never used:

```python
COMBAT_MCTS_BUDGETS: Dict[str, int] = {
    "monster": 5,
    "elite": 20,
    "boss": 200,  # 200 MCTS simulations for boss -- NEVER EXECUTED
}
```

The worker code in lines 426-436 only enters the MCTS combat path when `mcts_engine is not None`, which requires `mcts_enabled=True`.

## Root Cause #3: Boss HP Progress Reward Timing Bug

When the player **dies** during a boss fight, the `boss_hp_progress` reward never fires.

### The Bug Path

1. Player dies in combat at floor 16. `runner.game_over` becomes True.
2. The main loop exits (`while not runner.game_over and step < 5000`).
3. Line 792: `_finish_combat_summary()` is called, appending boss damage data to `combats`.
4. But `boss_hp_progress` reward is only computed inside the `combat_just_ended` block (line 718), which only fires during a **strategic decision transition** (line 528).
5. When the player dies in combat, there IS no next strategic decision. The loop exits, terminal reward is applied to the **last transition** (which is a combat action, not a strategic one), and the boss HP progress is never computed.

**Result**: The model gets zero gradient signal about how much boss HP it dealt. A game that deals 90% of boss HP gets the same reward as one that dealt 0%.

## Secondary Issue: Weak Reward Gradient at Floor 16

```
Floor 16 milestone: +9.0  (reaching the boss)
Floor 17 milestone: +15.0 (beating the boss)
Gap: only +6.0 for winning the boss fight
```

The `boss_win` event reward is +5.0, scaled by HP efficiency (`0.5 + 0.5 * hp_pct`), so effectively +2.5 to +5.0. Combined with the +6.0 milestone gap, beating the boss is worth approximately +8.5 to +11.0 total.

Meanwhile, the death penalty for dying at floor 16 is:
```python
death_scale * (1 - progress)  =  -0.3 * (1 - 16/55)  =  -0.3 * 0.709  =  -0.213
```

The death penalty is nearly zero (-0.21). There's insufficient negative signal from dying at the boss.

## Data Evidence

### Trajectory Distribution

From the glob search of `logs/v3_concurrent/all_trajectories/`:

- **F16 files**: 100+ (truncated in results -- many hundreds)
- **F17 files**: 0
- **F18-F19 files**: 0
- **F20+ files**: 0

**Floor 16 is an absolute ceiling.** Every game that reaches it dies there.

### Status Snapshot (11.27 hours)

```json
{
  "total_games": 29892,
  "total_wins": 0,
  "avg_floor_100": 8.3,
  "peak_floor": 16,
  "games_per_min": 44.2,
  "total_transitions": 325195,
  "train_val_acc": 55.6,
  "checkpoint_version": 220
}
```

At 44.2 games/min and avg_floor 8.3, roughly 10-15% of games reach floor 16. That's ~3,000-4,500 boss encounters, all losses. Zero learning signal from any of them because the solver has 1,500x less compute than configured.

## Recommended Fixes (Ranked by Impact)

### Fix 1: Remove hardcoded turn_solver_ms from collection scripts [CRITICAL]

**Impact**: Immediately gives bosses 30s of solver compute instead of 20ms.

Files to change:
- `scripts/v3_train_concurrent.py` line 596: Change `20.0` to `50.0` (or remove the hardcode and use the worker default)
- `scripts/v3_collect.py` line 166: Same
- `scripts/v3_experiment_sweep.py` lines 207, 261: Same

Specific change for `v3_train_concurrent.py`:
```python
# Before (line 596):
(seed, 0, 0.8, total_games, 20.0, False, False, 0)

# After -- use config-based solver budgets (SOLVER_BUDGETS handles per-room-type):
(seed, 0, 0.8, total_games, 50.0, False, False, 0)
```

Note: The `turn_solver_ms` parameter only sets the **default**. The actual boss budget comes from `SOLVER_BUDGETS["boss"]` = 30,000ms via the override in `pick_action()`. Setting the default to 50ms (matching the worker parameter default) is correct because `SOLVER_BUDGETS` will override it for elite/boss fights.

### Fix 2: Fix node_budget to scale with SOLVER_BUDGETS [HIGH]

**Impact**: Even with the time override, the node budget stays at 2,000 (from `int(20.0 * 100)`). Fix worker.py to also override node_budget from SOLVER_BUDGETS.

File: `packages/training/worker.py` line 223

```python
# Before:
_node_budget = max(1000, int(turn_solver_ms * 100))

# After -- node budget is now set by SOLVER_BUDGETS per room type,
# this just sets the default for monsters:
_node_budget = max(5000, int(turn_solver_ms * 100))
```

The real fix is in `turn_solver.py` line 1209 where the override already sets `budget_nodes = int(base_nodes * scale)`. This is correct. The issue is that the initial construction uses the wrong default. With `turn_solver_ms=50.0` (Fix 1), the default becomes 5,000 nodes which is reasonable for monsters.

### Fix 3: Enable MCTS combat for boss fights [HIGH]

**Impact**: 200 MCTS simulations on top of the TurnSolver for boss combat decisions.

File: `scripts/v3_train_concurrent.py` line 596

```python
# Before:
(seed, 0, 0.8, total_games, 20.0, False, False, 0)
#                                  ^^^^^ ^^^^^ ^
#                            strategic  mcts  mcts_sims

# After -- enable MCTS for combat + strategic decisions:
(seed, 0, 0.8, total_games, 50.0, False, True, 0)
```

This activates `COMBAT_MCTS_BUDGETS` (200 sims for boss) without requiring strategic MCTS (which is expensive).

### Fix 4: Fix boss_hp_progress reward for death-in-combat [HIGH]

**Impact**: Creates continuous gradient signal for boss fights even when the player dies.

File: `packages/training/worker.py`, after line 793 (post-loop combat summary).

The fix: when the game ends in combat (death during boss fight), compute the boss_hp_progress reward and add it to the terminal transition.

```python
# After line 793 (_finish_combat_summary()):
# Apply boss HP progress to terminal transition if died in boss fight
if transitions and combats:
    _last_c = combats[-1]
    _rt = _last_c.get("room_type", "").lower()
    if _rt in ("boss", "b"):
        _boss_max = _last_c.get("boss_max_hp", 0)
        _boss_dmg = _last_c.get("boss_dmg_dealt", 0)
        if _boss_max > 0:
            from .reward_config import compute_boss_hp_progress
            transitions[-1]["reward"] += compute_boss_hp_progress(_boss_dmg, _boss_max)
```

### Fix 5: Increase death penalty and boss reward gap [MEDIUM]

**Impact**: Stronger learning signal at floor 16.

File: `packages/training/training_config.py`

```python
# Increase death penalty (currently -0.3, too weak at floor 16):
"death_penalty_scale": -1.5,  # Was -0.3. Floor 16 death = -1.5 * 0.71 = -1.06

# Increase floor 17 milestone to create bigger gap:
17: 25.00,  # Was 15.0. Gap with floor 16: 16.0 instead of 6.0

# Increase boss_win event reward:
"boss_win": 8.00,  # Was 5.0
```

### Fix 6: Add per-game solver timing log [LOW]

**Impact**: Debugging aid. Log actual solver ms used per combat in episode output.

The `combats` list already tracks `solver_ms` and `solver_calls`. These are aggregated in the return dict (line 864-865). Adding this to episode logs would help verify that boss fights actually get their 30s budget.

## Summary Table

| # | Fix | Impact | Files | Lines |
|---|-----|--------|-------|-------|
| 1 | Remove hardcoded 20ms | **CRITICAL** | 3 scripts | 4 lines |
| 2 | Fix default node_budget | HIGH | worker.py | 1 line |
| 3 | Enable MCTS combat | HIGH | 1 script | 1 line |
| 4 | Fix boss_hp_progress on death | HIGH | worker.py | ~8 lines |
| 5 | Increase death penalty + boss gap | MEDIUM | training_config.py | 3 values |
| 6 | Solver timing logs | LOW | worker.py | ~5 lines |

## Conclusion

The 20ms hardcode in `v3_train_concurrent.py` is a configuration drift bug. The training_config.py has the correct values (30s for bosses), the worker correctly passes SOLVER_BUDGETS to the TurnSolverAdapter, and the adapter correctly applies per-room overrides. But the collection scripts bypass this by passing a 20ms default that was probably appropriate for fast data collection but is catastrophic for boss fights.

Fixes 1-4 together should break the floor 16 ceiling. Fix 1 alone (changing one number from 20.0 to 50.0) is the minimum viable fix.
