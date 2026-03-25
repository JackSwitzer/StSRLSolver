# Codex Optimization Audit: Slay the Spire RL Pipeline

## Task

Perform a comprehensive performance audit of this Slay the Spire reinforcement learning project. The system trains a Watcher (A20) bot using behavioral cloning + PPO with MCTS tree search. Despite 12k+ games of training, it has **zero wins** and GPU utilization sits at **18%**. Your job is to identify concrete bottlenecks, propose fixes, and estimate their impact.

---

## 1. Project Overview

**Goal**: Train a bot that wins Slay the Spire as Watcher on Ascension 20 with >96% win rate.

**Hardware**: M4 Mac Mini, 24GB unified RAM, 10 GPU cores (Metal/MPS), 10 CPU cores.

**Architecture**:

```
                  ┌──────────────────────────────────────────────────┐
                  │            CONCURRENT TRAINING LOOP              │
                  │                                                  │
┌─────────────────┤  Training Thread (MPS GPU)                       │
│  PyTorch MPS    │    - BC on trajectory .npz files                 │
│  batch=2048     │    - AdamW, cosine LR, per-head multipliers      │
│  10 epochs/cyc  │    - Validates on 15% holdout                    │
│  ~40s active    │    - Saves checkpoint every cycle                │
└─────────────────┤                                                  │
                  │  Collection Thread (CPU + MLX)                   │
┌─────────────────┤    - MLX inference server (batch timeout 10ms)   │
│  10 workers     │    - 10 worker processes via mp.Pool             │
│  spawn context  │    - Each worker: game loop + TurnSolver         │
│  32 games/batch │    - Saves .npz trajectories to disk             │
└─────────────────┤    - Resyncs model weights every 500 games       │
                  └──────────────────────────────────────────────────┘
```

**Two-model architecture**:
- **StrategicNet** (18M params): 8x1024 residual trunk + policy/value/floor/act heads. Handles path, card pick, rest, shop, event decisions. Input: 480-dim RunStateEncoder.
- **CombatNet** (small, ~200K params): 298-dim input, 3-layer MLP. Predicts fight win probability for TurnSolver leaf evaluation.
- **TurnSolver**: Adaptive DFS/beam search for within-turn card play. Multi-turn lookahead for boss/elite.
- **MCTS**: AlphaZero-style for both combat and strategic decisions. Budgets scale by floor/phase/room type.

---

## 2. Current Metrics (as of 12k+ games)

| Metric              | Value     | Target     |
|---------------------|-----------|------------|
| Validation accuracy | 55.9%     | 70%+       |
| Training accuracy   | ~70%      | --         |
| Average floor       | 8.9       | 55 (win)   |
| Peak floor          | ~16       | 55         |
| Total wins          | 0         | >96% WR    |
| GPU utilization     | 18%       | >50%       |
| Games/minute        | ~2-3      | 10+        |
| Avg game duration   | ~25s      | <10s       |

**The critical wall is Floor 16** (Act 1 boss). The bot rarely survives the boss fight, suggesting the solver is either not computing enough for bosses or the strategic decisions leading up to the boss are suboptimal.

---

## 3. Known Bottlenecks

### 3.1 Hardcoded Defaults Override Config (CRITICAL BUG)

**File**: `scripts/v3_train_concurrent.py`, line 596

The collection thread calls `_play_one_game` with **hardcoded arguments** that bypass `training_config.py`:

```python
pool.apply_async(
    _play_one_game,
    (seed, 0, 0.8, total_games, 20.0, False, False, 0),
)
```

Cross-referencing with the function signature in `packages/training/worker.py` line 189:

```python
def _play_one_game(
    seed: str,
    ascension: int,          # 0  -- should be 20 for A20
    temperature: float,      # 0.8
    total_games: int = 0,
    turn_solver_ms: float = 50.0,  # 20.0 passed -- overrides the 50ms default
    strategic_search: bool = False, # Disabled
    mcts_enabled: bool = False,     # MCTS completely disabled
    mcts_card_sims: int = 0,       # No MCTS sims
)
```

**Issues**:
1. `ascension=0` -- training on A0 when the target is A20. Enemy damage, HP pools, and encounter difficulty are drastically different.
2. `turn_solver_ms=20.0` -- only 20ms per combat turn. Config says `SOLVER_BUDGETS["boss"] = (30_000.0, 200_000, 600_000)` but the TurnSolverAdapter is constructed with `time_budget_ms=20.0`, which becomes the default for all fights. The per-room-type budget scaling in lines 1198-1209 of `turn_solver.py` does override this for boss/elite, but the initial node budget is `max(1000, int(20.0 * 100))` = 2000 nodes -- much less than the 200K in config.
3. `mcts_enabled=False` -- all AlphaZero MCTS for strategic decisions is disabled.
4. `strategic_search=False` -- value-head per-option evaluation is disabled.

**Impact**: The bot is training on a much easier difficulty with minimal search. This likely explains why it plateaus -- it learns A0 patterns that don't transfer to A20.

### 3.2 GPU Utilization at 18% (CPU-bound game simulation)

**File**: `scripts/v3_train_concurrent.py`

The concurrent architecture has:
- Training thread: GPU active ~40s per cycle (batch=2048, 10 epochs)
- Collection thread: GPU idle for ~180s while collecting 500+ games

The GPU is idle during collection because:
1. Game simulation runs in Python on CPU (10 workers via `mp.Pool`)
2. MLX inference is the only GPU work during collection, but it's tiny (480-dim input, ~15ms batch timeout)
3. The training thread completely deallocates GPU memory between cycles (line 484: `del model, optimizer...`)

### 3.3 Python Engine is the Bottleneck

**File**: `packages/training/worker.py`

Each game takes ~25 seconds because:
1. The Python game engine (`packages/engine/`) runs the full game loop in pure Python
2. `CombatEngine.copy()` is called for every TurnSolver node expansion (Python deepcopy)
3. Card effect resolution, damage calculation, and enemy AI all run in Python
4. The TurnSolver (beam/DFS search) does hundreds-to-thousands of `copy() + execute_action()` calls per turn

**Profiling target**: The `combat_solver_ms` field in game results tracks solver time. Boss fights with `SOLVER_BUDGETS["boss"] = (30_000.0, ...)` could theoretically spend 30s per turn in the solver.

### 3.4 BC Overfitting (70% train / 55.9% val)

**File**: `scripts/v3_train_concurrent.py`, lines 378-484

The training loop shows classic overfitting:
- Train accuracy ~70%, validation accuracy ~55.9%
- 15% gap suggests the model memorizes training trajectories
- Batch size 2048 with only 10 epochs per cycle may not be enough regularization
- The model re-initializes from the latest checkpoint every cycle (line 340: `model = make_model(device)`) then loads weights -- this resets optimizer state

### 3.5 Evaluation Floor Disconnect

Despite 55.9% BC accuracy, the average floor is only 8.9. This suggests:
- The model makes correct predictions on average but fails at critical moments (boss fights, key card picks)
- Floor 16 requires near-perfect play for several consecutive decisions
- A single bad card pick early in the run can doom the entire run

---

## 4. Key Files for Investigation

All paths relative to project root (`/Users/jackswitzer/Desktop/SlayTheSpireRL/`):

### Training Pipeline
| File | Description | Lines |
|------|-------------|-------|
| `packages/training/training_config.py` | Single source of truth for all hyperparameters | 266 |
| `packages/training/worker.py` | Game worker -- plays one game, returns transitions | 886 |
| `packages/training/turn_solver.py` | TurnSolver + MultiTurnSolver + TurnSolverAdapter | ~1300 |
| `packages/training/strategic_net.py` | StrategicNet (18M params, 8x1024 residual) | 240 |
| `packages/training/combat_net.py` | CombatNet (298-dim in, 256 hidden, 3 layers) | 240 |
| `packages/training/state_encoders.py` | RunStateEncoder (480-dim) + CombatStateEncoder (298-dim) | 649 |
| `packages/training/inference_server.py` | MLX batch inference server, queue-based IPC | ~400 |
| `packages/training/shared_inference.py` | Shared memory zero-copy inference (Apple Silicon) | ~200 |
| `packages/training/mlx_inference.py` | MLX model mirroring PyTorch StrategicNet | ~300 |
| `packages/training/strategic_trainer.py` | PPO + GAE + OPR + auxiliary losses | ~500 |
| `packages/training/training_runner.py` | Main orchestrator (COLLECT -> TRAIN -> SYNC) | ~800 |
| `packages/training/reward_config.py` | Reward shaping, PBRS, milestone rewards | ~200 |

### Scripts
| File | Description |
|------|-------------|
| `scripts/v3_train_concurrent.py` | **Active training script** -- concurrent BC + collection | 802 lines |
| `scripts/training.sh` | Shell wrapper for training commands |

### Rust Engine (not merged)
| File | Description |
|------|-------------|
| `packages/engine-rs/Cargo.toml` | Rust crate config -- PyO3 bindings, opt-level 3, thin LTO |
| `packages/engine-rs/src/lib.rs` | Module entry: exposes `RustCombatEngine`, `PyCombatState`, `PyAction` |
| `packages/engine-rs/src/engine.rs` | Core combat engine -- turn loop, card play, damage, stances (766 lines) |
| `packages/engine-rs/src/state.rs` | CombatState, EnemyCombatState, Entity, Stance |
| `packages/engine-rs/src/cards.rs` | CardRegistry, CardDef, card effects |
| `packages/engine-rs/src/damage.rs` | Damage/block calculation with all modifiers |
| `packages/engine-rs/src/actions.rs` | Action enum + PyO3 bindings |
| `packages/engine-rs/src/powers.rs` | Power/debuff application (metallicize, poison, ritual) |
| `packages/engine-rs/benches/` | Criterion benchmarks for combat simulation |

### Python Engine (source of truth)
| File | Description |
|------|-------------|
| `packages/engine/` | Full Python game engine (6154+ tests, 100% Java parity) |
| `packages/engine/combat_engine.py` | Python CombatEngine used by TurnSolver |
| `packages/engine/game.py` | GameRunner -- full game loop (map, events, combat, shop, rest) |

---

## 5. Optimization Targets

### Target A: Fix Hardcoded Defaults in v3_train_concurrent.py

**Priority**: CRITICAL -- this is likely the #1 reason for 0 wins.

Investigate `scripts/v3_train_concurrent.py` line 596 and fix the hardcoded args:

```python
# Current (broken):
(seed, 0, 0.8, total_games, 20.0, False, False, 0)

# Should reference training_config:
(seed, 20, TEMPERATURE, total_games, SOLVER_BUDGETS["monster"][0],
 True, True, MCTS_BUDGETS["card_pick"])
```

**Specific asks**:
1. Identify all places where `_play_one_game` is called with hardcoded values
2. Replace with references to `training_config.py` constants
3. Verify the TurnSolverAdapter actually receives and applies the correct budgets for boss fights
4. Trace the budget flow: `training_config.SOLVER_BUDGETS` -> `worker._play_one_game(turn_solver_ms=...)` -> `TurnSolverAdapter.__init__` -> `TurnSolver.default_time_budget_ms` -> `solve_turn()` time limit

### Target B: Profile the Concurrent Training Pipeline

**Goal**: Determine where wall-clock time is spent.

1. Instrument `_play_one_game` to break down time by phase:
   - Game engine initialization (`GameRunner(seed=...)`)
   - Combat turns (total solver time is already tracked in `combat_solver_ms`)
   - Strategic decisions (inference roundtrip time)
   - State encoding (`RunStateEncoder.encode()`)
   - Inference server latency (queue wait time)
2. Instrument the training thread:
   - Data loading time (`.npz` file reads)
   - Forward/backward pass time
   - GPU memory transfer time
   - Checkpoint save time
3. Identify the ratio: what % of wall time is training vs collection vs idle?

### Target C: Evaluate Rust Engine Integration

**Goal**: Determine if the Rust engine can provide 10x+ simulation speedup for combat.

The Rust engine exists at `packages/engine-rs/` with:
- PyO3 bindings (Python-callable)
- Core combat loop (start_combat, play_card, end_turn, get_legal_actions)
- Damage/block/stance/power math
- `clone_for_mcts()` method for MCTS tree expansion
- 516 tests on a separate branch (not merged to main)

**Specific asks**:
1. Audit the Rust engine's feature coverage vs the Python engine:
   - Which cards are supported? (Check `packages/engine-rs/src/cards.rs`)
   - Which powers/effects are missing?
   - What edge cases fall back to Python?
2. Estimate the integration effort:
   - Can TurnSolver use `RustCombatEngine` as a drop-in replacement for `CombatEngine`?
   - What adapter code is needed?
   - Are the Action types compatible?
3. Benchmark estimate: Python `CombatEngine.copy() + execute_action()` vs Rust `clone_for_mcts() + take_action()`
4. Propose a hybrid approach: Rust for the hot path (TurnSolver node expansion), Python for edge cases

### Target D: GPU Utilization Plan (>50%)

**Goal**: Keep the GPU busy more than 50% of the time.

Current bottleneck: the training thread only uses GPU for ~40s per cycle, then waits for collection. The collection thread uses MLX (also GPU) for inference but at very low utilization.

**Investigate**:
1. **Overlap training and collection on GPU**: Can MPS training and MLX inference coexist? (They should on Apple Silicon unified memory -- the project claims this works.)
2. **Larger batch training**: Current batch_size=2048. Can we increase to 8192 or 16384 given 24GB unified RAM? An 18M param model in fp32 uses ~72MB. Activations for batch=8192 with 480-dim input and 1024 hidden would use ~50-100MB. Plenty of room.
3. **Mixed precision training**: The model trains in fp32. Switching to fp16/bf16 on MPS could nearly double throughput.
4. **Continuous training**: Instead of "train 10 epochs, wait, reload", train continuously on a growing replay buffer with streaming data loading.
5. **More frequent training cycles**: Reduce the 500-game resync interval. Train on new data as it arrives.

### Target E: Training Loop Improvements

**Investigate these specific improvements**:

1. **Optimizer state persistence**: The training thread creates a new model+optimizer every cycle (line 340-394). This resets momentum/variance buffers. Instead, keep the optimizer alive across cycles and just add new data.

2. **Learning rate**: BC uses `lr=3e-4` (hardcoded on line 379) but `training_config.py` says `LR_BASE=3e-5`. This is a 10x discrepancy.

3. **Value head target**: The value head is trained with `v_loss = F.mse_loss(out["value"], floor_t[batch])` where `floor_t` is `final_floor / 55.0`. This means the value head predicts "how far will I get" rather than expected return. This is fine for BC but may conflict with PPO's value function later.

4. **Data loading**: `load_all_trajectories()` does `Path("logs").rglob("traj_*.npz")` every cycle, which walks the entire log directory tree. This could be slow with thousands of files.

5. **Auxiliary losses**: The BC training only uses policy loss + value loss. The StrategicNet also has floor_pred and act_completion heads that aren't being trained in the concurrent script.

---

## 6. Specific Deliverables for the Codex PR

### Must-have (fix bugs):
1. Fix the hardcoded `_play_one_game` call in `v3_train_concurrent.py` to use `training_config` values
2. Fix the LR discrepancy (3e-4 vs 3e-5)
3. Add logging to confirm solver budgets are actually applied for boss fights

### Should-have (performance):
4. Profile instrumentation for `_play_one_game` (time breakdown by phase)
5. Persistent optimizer state across training cycles
6. Increase batch size from 2048 to 8192
7. Train auxiliary heads (floor_pred, act_completion) in BC loop

### Nice-to-have (architecture):
8. Rust engine integration plan with effort estimate
9. Mixed precision training support
10. Continuous training loop (no cycle-based reloading)

---

## 7. Architecture Reference

### StrategicNet (18M params)
```
Input: 480-dim (RunStateEncoder)
  -> Linear(480, 1024) + LayerNorm + ReLU
  -> 8x ResidualBlock(1024)
       each: Linear(1024,1024) -> ReLU -> Linear(1024,1024) -> LayerNorm + skip
  -> Policy head: Linear(1024, 256) -> ReLU -> Linear(256, 512)
  -> Value head: Linear(1024, 64) -> ReLU -> Linear(64, 1)
  -> Floor head: Linear(1024, 64) -> ReLU -> Linear(64, 1)
  -> Act head: Linear(1024, 64) -> ReLU -> Linear(64, 3) -> Sigmoid
```

### RunStateEncoder (480-dim)
```
  6   HP/resources (hp_ratio, max_hp, gold, floor, act, ascension)
  3   Keys (ruby, emerald, sapphire)
 16   Deck functional aggregate (avg effects + composition)
181   Relic binary flags
 20   Potion slots (5 x 4 functional dims)
 21   Map lookahead (3 rows x 7 room types)
  4   Progress (combats_won, elites, bosses, boss_id)
  3   HP deficit + is_boss + is_elite
  6   Phase type one-hot (path/card/rest/shop/event/other)
220   Action encoding (10 slots x 22 dims per action)
---
480   total
```

### CombatStateEncoder (298-dim)
```
  9   Energy, block, turn, hand_size, draw/discard/exhaust, stance
  1   Mantra
 40   Active powers (20 x 2: present + amount)
180   Hand cards (10 x 18 effect dims)
 60   Enemies (5 x 12: hp, max_hp, block, move_dmg, move_hits, alive, debuffs)
  6   Draw pile summary
  2   Discard summary
---
298   total
```

### TurnSolver Strategy Selection
```
Estimated nodes < 300   -> Exact DFS with alpha-beta pruning
300 <= nodes < 5000     -> Best-first search with pruning
nodes >= 5000           -> Beam search with reserved setup slots
```

### Solver Budget Flow
```
training_config.SOLVER_BUDGETS = {
    "monster": (50ms, 5K nodes, 300K ms cap),
    "elite": (2s, 50K nodes, 600K ms cap),
    "boss": (30s, 200K nodes, 600K ms cap),
}

worker._play_one_game(turn_solver_ms=X)
  -> TurnSolverAdapter.__init__(time_budget_ms=X, node_budget=X*100)
     -> self._solver = TurnSolver(time_budget_ms=X, node_budget=X*100)
     -> self._multi_turn = MultiTurnSolver(inner=TurnSolver(500ms, 50K))
  -> pick_action(room_type="boss")
     -> reads self._solver_budgets[room_type] -> (30000, 200000, 600000)
     -> scales by enemy HP / 100
     -> sets self._solver.default_time_budget_ms = 30000 * scale
     -> calls self._multi_turn.solve(engine) for boss/elite
     -> falls back to self._solver.solve_turn(engine) for monster
```

**BUG**: When `turn_solver_ms=20.0` is passed from v3_train_concurrent.py, the initial node budget is `max(1000, int(20.0 * 100))` = 2000 nodes. Even though the budget is later overridden for boss fights via `self._solver_budgets`, the MultiTurnSolver's inner_solver is constructed with a hardcoded `TurnSolver(500ms, 50K)` on line 1131. The main solver's default of 2K nodes is only used for monster fights -- boss fights get the multi-turn solver. Verify this flow end-to-end.

### MCTS Budget Flow (currently disabled in v3_train_concurrent.py)
```
training_config.COMBAT_MCTS_BUDGETS = {
    "monster": 5 sims,
    "elite": 20 sims,
    "boss": 200 sims,
}

training_config.MCTS_BUDGETS = {
    "card_pick": 200 sims,
    "path": 50, "rest": 20, "shop": 20, "event": 30,
}

MCTS_FLOOR_MULTIPLIERS = {0: 10x, 1: 5x, 15: 3x, 16: 5x, ...}
MCTS_PHASE_MULTIPLIERS = {"card_pick": 2x, "rest": 1.5x, ...}
```

### Rust Engine Coverage
```
Supported:
  - Basic cards: Strike, Defend, Eruption, Vigilance (+ upgraded)
  - Damage/block with all modifiers (strength, dex, vulnerable, weak, frail)
  - Stance system: Wrath (2x damage in/out), Calm (+2 energy on exit), Divinity (auto-exit)
  - Turn loop: draw, play, discard, enemy attack, block decay
  - Enemy AI: damage + block + debuff application
  - Powers: metallicize, plated armor, poison, ritual
  - PyO3 bindings: clone_for_mcts(), get_legal_actions(), take_action()

Missing (falls back to Python):
  - Scry, X-cost cards, complex card effects
  - Most Watcher-specific cards beyond basics
  - Full potion effects (stub only)
  - Relic triggers during combat
  - Orb system (not relevant for Watcher)
```

---

## 8. Reproduction Commands

```bash
# Run tests
uv run pytest tests/ -q

# Start concurrent training (current)
nohup uv run python scripts/v3_train_concurrent.py > logs/v3_concurrent_stdout.log 2>&1 &

# Check training status
cat logs/v3_concurrent/status.json | python -m json.tool

# Build Rust engine
export PYO3_PYTHON=.venv/bin/python3
cargo build --release --manifest-path packages/engine-rs/Cargo.toml

# Run Rust benchmarks
cargo bench --manifest-path packages/engine-rs/Cargo.toml

# Run Rust tests
cargo test --lib --manifest-path packages/engine-rs/Cargo.toml
```

---

## 9. Success Criteria

After this audit, the PR should contain:

1. **Bug fixes** for hardcoded values in `v3_train_concurrent.py` (ascension, solver budget, MCTS enable)
2. **Profiling data** showing where time is spent in `_play_one_game`
3. **Concrete recommendations** with estimated impact:
   - "Fixing ascension from 0 to 20 will make training match target difficulty"
   - "Enabling MCTS will add ~Xms per strategic decision but improve card pick quality"
   - "Increasing batch size from 2048 to 8192 will increase GPU utilization from 18% to ~Y%"
   - "Rust engine for TurnSolver would speed up combat simulation by ~Z x"
4. **A plan** for getting GPU utilization above 50%
5. **If feasible**: a working Rust engine integration for TurnSolver node expansion

The single highest-impact change is likely fixing the hardcoded `ascension=0` and `mcts_enabled=False`. Everything else is secondary until the bot is actually training on the right difficulty.
