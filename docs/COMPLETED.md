# Completed Work

Reverse chronological. PR references link to GitHub.

## 2026-03-25: V3 Clean Merge + Git Consolidation
**PRs**: #62-69 merged to main

### Git Consolidation
- Merged all open PRs (#62-69) into clean main
- Closed stale worktree branches
- PR #69: final clean merge of all v3 work

### V3 Bug Fixes (PR #67 — Codex audit)
- Fixed value head output mismatch (was returning wrong dimension)
- Removed silent `except Exception: pass` blocks (3 critical bugs across 255k games)
- Fixed double-seeded RNG initialization
- Fixed strategic search same-state loop
- Fixed collect overshoot (games exceeding target count)
- Fixed shutdown wedge (workers hanging on exit)
- Fixed slot retry logic in inference server
- Fixed status.json write race condition
- Fixed distillation repeat bug
- Fixed abort criteria (premature training termination)

### BC Pretrain Pipeline
- Built pretrain-from-trajectories pipeline
- 12,942 transitions baseline, 43.7% accuracy after 10 epochs
- Per-head learning rates: trunk=1x, policy=2x, value=3x, aux=1x
- Scripts: `v3_pretrain.py`, `v3_pretrain_gpu.py`

### CombatNet
- 298-dim input combat state encoder
- 92% validation accuracy on 162k combat positions
- 70/30 neural/heuristic blend in TurnSolver
- Saved to `logs/active/combat_net.pt`

### 4-Experiment Sweep
- Experiment A (baseline): avg floor 6.27
- Experiment B (reward tuned): avg floor 6.35
- **Experiment C (BC + PPO): avg floor 8.11** (winner)
- Experiment D (incomplete)
- Automated sweep via `v3_experiment_sweep.py`

### Overnight Concurrent Training
- 220 training cycles completed
- val_acc 55.9%, avg floor 8.9
- Peak floor 16 (Act 1 boss)
- 0 wins — floor 16 wall identified

### Floor 16 Root Cause Analysis
- All games die at Act 1 boss
- Solver budget insufficient for boss fights
- Boss HP progress reward captured during combat (not after)
- Analysis in `docs/research/floor16-root-cause.md`

### Action Masking Fix
- Fixed action mask contract in gym environment
- Prevented model from selecting invalid actions

### Diagnostic Metrics
- Added `time.monotonic` timing throughout pipeline
- Episode logging with per-floor diagnostics
- DiagnosticsCharts, SweepComparison, CardPickSummary app views

### Training Infrastructure
- `training_config.py` as single source of truth
- Config-driven scripts (no hardcoded params)
- `pause.sh` for graceful training pause
- Auto-archive with timestamps (never rm -rf)

## 2026-03-19: AlphaZero MCTS Weekend Run
**PR**: #66

- AlphaZero-style MCTS for strategic decisions (500 sims)
- 15 workers, shared memory, pure search scoring
- Dynamic compute budgets per decision complexity
- 1h game timeout (was 120s — boss fights need 5+ min)
- TurnSolver Wrath penalty fix (-60 was #1 blocker for 235k+ games)
- App: boss deep-dive view, 500ms polling, per-config metrics
- 6154 tests passing

## 2026-03-17: Training V2 + Reward System
**PR**: #65

- PPO + GAE training loop with OPR auxiliary losses
- Reward v12: 3x milestones, -0.3 death, 1.5x PBRS
- Boss solver fix (sim.state.player.hp, no tick loop)
- Exception safety audit (13 blocks with logging)
- BC warmup in strategic_trainer.py
- MCTS strategic search in worker.py
- Merl seeds (12 A20 expert seeds in seed_pool.py)
- 6076 tests passing

## Earlier Work
- Python game engine with 100% Java parity (Watcher)
- Rust combat engine with PyO3 bindings
- SwiftUI macOS monitoring dashboard
- WebSocket server for live metrics
- Full test suite covering cards, powers, relics, events, RNG
