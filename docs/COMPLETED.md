# Completed Work Units

Auto-generated from `docs/work_units/archive/` YAML frontmatter. Updated via `/pr`.

## Engine Parity (completed 2026-03)

| Work Unit | Completed | Notes |
|-----------|-----------|-------|
| [cards-watcher](work_units/archive/granular-cards-watcher.md) | 2026-03-15 | 7/7 (100%) |
| [cards-ironclad](work_units/archive/granular-cards-ironclad.md) | 2026-03-15 | 61/62 (98%) |
| [cards-silent](work_units/archive/granular-cards-silent.md) | 2026-03-15 | 60/61 (98%) |
| [cards-shared](work_units/archive/granular-cards-shared.md) | 2026-03-15 | 5/5 (100%) |
| [cards-defect](work_units/archive/granular-cards-defect.md) | 2026-03-15 | 74/75 (98%) |
| [orbs](work_units/archive/granular-orbs.md) | 2026-03-10 | 28/28 (100%) |
| [potions](work_units/archive/granular-potions.md) | 2026-03-10 | 34/34 (100%) |
| [rewards](work_units/archive/granular-rewards.md) | 2026-03-10 | 4/4 (100%) |

## Training V3 (completed 2026-03-25)

PRs #62-69 merged to main.

### Bug Fixes (PR #67 — Codex audit)
- Value head output mismatch, silent except blocks, double-seeded RNG
- Strategic search same-state loop, collect overshoot, shutdown wedge
- Slot retry, status.json race, distillation repeat, abort criteria

### BC Pretrain Pipeline
- Pretrain-from-trajectories pipeline (12,942 transitions, 43.7% accuracy)
- Per-head LR: trunk=1x, policy=2x, value=3x, aux=1x

### CombatNet
- 298-dim input, 92% val accuracy on 162k positions
- 70/30 neural/heuristic blend in TurnSolver

### 4-Experiment Sweep
- A(6.27), B(6.35), **C(BC+PPO)=8.11 winner**, D(incomplete)

### Overnight Training
- 220 cycles, val_acc 55.9%, avg floor 8.9, peak F16, 0 wins

### Infrastructure
- training_config.py as single source of truth
- Config-driven scripts, pause.sh, auto-archive

## AlphaZero MCTS Weekend (2026-03-19, PR #66)
- MCTS 500 sims for strategic decisions, 15 workers
- TurnSolver Wrath penalty fix (235k+ games affected)
- Dynamic compute budgets, 1h game timeout

## Training V2 + Rewards (2026-03-17, PR #65)
- PPO + GAE + OPR training loop
- Reward v12, BC warmup, MCTS strategic search
- Merl seeds (12 A20 expert seeds)

## Earlier
- Python game engine (100% Java parity, Watcher)
- SwiftUI macOS monitoring dashboard
- WebSocket server for live metrics
- Full test suite (6227+ tests)
