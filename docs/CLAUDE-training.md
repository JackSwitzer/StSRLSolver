# Training Architecture Reference

## Pipeline Overview
```
BC Pretrain (offline)  →  PPO Online (COLLECT → TRAIN → SYNC loop)
                              ↓
                    CombatNet (70/30 neural/heuristic)
                    TurnSolver (beam search, dynamic budget)
                    StrategicNet (policy + value + aux heads)
```

## Neural Networks

### StrategicNet (~18M params)
- **Architecture**: Residual trunk (8 blocks x 1024 hidden) + multi-head output
- **Input**: RunStateEncoder (480-dim) — HP, gold, deck composition, relics, map, floor
- **Heads**: policy (action logits), value (scalar), auxiliary (floor/act prediction)
- **Per-head LR**: trunk=1x, policy=2x (3e-5 * 2), value=3x, auxiliary=1x
- **File**: `packages/training/strategic_net.py`

### CombatNet
- **Input**: CombatStateEncoder (298-dim) — player HP/block/energy, hand, enemies, powers
- **Output**: combat state evaluation (scalar)
- **Val accuracy**: 92% on 162k positions
- **Blend**: 70% neural / 30% heuristic in TurnSolver
- **File**: `packages/training/combat_net.py`

## Algorithms

| Algorithm | Status | File | Notes |
|-----------|--------|------|-------|
| **PPO** | Active | `strategic_trainer.py` | GAE + OPR aux losses, clip=0.2 |
| **IQL** | Config exists, not wired | `iql_trainer.py` | Offline RL path, expectile=0.7 |
| **GRPO** | Config exists, not wired | `grpo_trainer.py` | Falls back to PPO currently |
| **MCTS** | Active (strategic) | `strategic_mcts.py` | AlphaZero-style, UCB=1.414 |

## Solver Configuration
```
Room type    Base budget    Node cap      Time cap
monster      50ms           5,000         5 min
elite        2,000ms        50,000        10 min
boss         30,000ms       200,000       10 min
```
Budget scales with total enemy HP: `budget_ms = base * max(1, total_hp / 100)`

## Training Loop
1. **COLLECT**: Workers play games using current model + solver
2. **TRAIN**: PPO on collected trajectories (4 epochs, batch=256)
3. **SYNC**: Push weights to inference server
4. Repeat (30 steps/phase, 500 games/collect)

## Experiment History (2026-03-25)
| Config | Approach | Avg Floor | Notes |
|--------|----------|-----------|-------|
| A | Baseline PPO | 6.27 | Control |
| B | Reward tuned (v12) | 6.35 | Minimal improvement |
| **C** | **BC pretrain + PPO** | **8.11** | **Winner** |
| D | Incomplete | — | Aborted |

Best overnight run: 220 cycles, val_acc 55.9%, avg floor 8.9, peak F16.

## Known Issues
- **Floor 16 wall**: All games die at Act 1 boss. Solver budget may still be insufficient for boss decision quality.
- **Value head**: Pretrain loss 47k due to unnormalized returns in `_pretrain_from_trajectories`
- **EndTurn bug**: Solver ends turns with 3 energy + 5 playable cards (10ms budget insufficient)
- **GRPO**: Falls back to PPO (full rollout collection not wired)
- **IQL**: Config exists but dispatch not wired in training_runner

## Key Files by Role

### Core Pipeline
- `training_config.py` — All hyperparameters (single source of truth)
- `training_runner.py` — COLLECT → TRAIN → SYNC orchestrator
- `worker.py` — Game workers (10 parallel)

### Networks + Encoders
- `strategic_net.py` — StrategicNet (18M, 1024h, 8 blocks)
- `combat_net.py` — CombatNet (298-dim)
- `state_encoders.py` — RunStateEncoder (480) + CombatStateEncoder (298)
- `mlx_inference.py` — MLX mirror for Apple Silicon inference

### Trainers + Search
- `strategic_trainer.py` — PPO + GAE + OPR
- `iql_trainer.py` — Implicit Q-Learning (offline)
- `grpo_trainer.py` — Group Relative Policy Optimization
- `turn_solver.py` — Beam search combat solver
- `strategic_mcts.py` — AlphaZero MCTS for strategic decisions

### Infrastructure
- `inference_server.py` — MLX batch inference (Metal GPU)
- `shared_inference.py` — Shared inference utilities
- `replay_buffer.py` — Experience replay
- `episode_log.py` — Episode logging + diagnostics
- `reward_config.py` — Hot-reloadable reward adapter
- `sweep_config.py` — Experiment configurations
- `seed_pool.py` — 12 Merl A20 expert seeds
- `gym_env.py` — Gym-compatible wrapper
- `offline_data.py` — Offline dataset loading
- `conquerer.py` — Game completion tracker
