# Slay the Spire RL Project

Build a bot that wins Slay the Spire (Watcher, A20, >96% WR) via RL + tree search.

## Structure
```
packages/engine/       # Python game engine (source of truth, 100% Java parity)
packages/engine-rs/    # Rust combat engine (PyO3 bindings, solver speedup)
packages/training/     # RL pipeline
  training_config.py   # ← SINGLE SOURCE OF TRUTH for all params
  training_runner.py   # Orchestrator (COLLECT → TRAIN → SYNC)
  worker.py            # Game worker (10 parallel, plays via solver + model)
  strategic_net.py     # StrategicNet (PyTorch, ~18M params, hidden=1024)
  strategic_trainer.py # PPO + GAE + OPR + auxiliary losses
  strategic_mcts.py    # AlphaZero-style MCTS for strategic decisions
  inference_server.py  # MLX batch inference on Metal GPU
  state_encoders.py    # RunStateEncoder (480-dim) + CombatStateEncoder
  turn_solver.py       # Beam search combat solver (dynamic budgets)
  combat_net.py        # CombatNet (298-dim input, 92% val accuracy)
  offline_data.py      # Offline dataset loading (trajectories + combat)
  iql_trainer.py       # Implicit Q-Learning (offline RL path)
  grpo_trainer.py      # Group Relative Policy Optimization
  reward_config.py     # Thin adapter over training_config (hot-reloadable)
  sweep_config.py      # Experiment configs (references training_config)
  seed_pool.py         # Seed rotation + 12 Merl A20 expert seeds
  mlx_inference.py     # MLX model (mirrors PyTorch for Apple Silicon)
  shared_inference.py  # Shared inference utilities
  replay_buffer.py     # Experience replay buffer
  episode_log.py       # Episode logging + diagnostics
  gym_env.py           # Gym-compatible environment wrapper
  conquerer.py         # Conquerer (game completion tracker)
packages/app/          # SwiftUI macOS monitoring dashboard
packages/server/       # WebSocket server for dashboard
tests/                 # 6227+ tests (pytest)
scripts/               # Shell + Python training scripts
  training.sh          # Start/stop/status for training
  services.sh          # Background services management
  pause.sh             # Graceful pause for running training
  app.sh               # Build + launch macOS app
  v3_pretrain.py       # BC pretrain on trajectory data
  v3_pretrain_gpu.py   # GPU-accelerated pretrain
  v3_collect.py        # Data collection runner
  v3_train_concurrent.py  # Concurrent collect+train loop
  v3_train_overnight.py   # Overnight training runner
  v3_experiment_sweep.py  # Automated experiment sweep
  v3_sweep.py          # Quick sweep launcher
  v3_1h_test.py        # 1-hour smoke test
  v3_overnight.py      # Long overnight run
docs/                  # vault/ (game mechanics), research/, TODO.md
```

## Commands
```bash
uv run pytest tests/ -q                    # All tests (6227+)
bash scripts/training.sh start --games N   # Start training
bash scripts/training.sh status            # Live metrics
bash scripts/training.sh stop              # Graceful shutdown
bash scripts/pause.sh                      # Graceful pause
bash scripts/app.sh                        # Build + launch macOS app
export PYO3_PYTHON=.venv/bin/python3 && cargo test --lib --manifest-path packages/engine-rs/Cargo.toml
```

## Workflow Rules

### Subagents
- **Opus 4.6**: ALL tasks. Never use older models.
- **Haiku**: Only for quick sub-sub-agent searches within Opus agents.
- Delegate heavily. Use worktrees for parallel work.

### Merging (CRITICAL)
- **NEVER** copy files from worktrees to main. Use `/ship` or `/merge-worktrees`.
- Every merge gets pre-merge audit (import chains, dead code, config consistency).
- Commit early on worktrees. Uncommitted changes get destroyed.
- See memory: `feedback_merge_properly.md`, `feedback_never_copy_merge.md`

### Error Policy
- No bare `except Exception: pass` — MUST log at WARNING+.
- Allowed: `except queue.Empty`, `except KeyboardInterrupt`, documented expected exceptions.
- Silent exception swallowing caused 3 critical bugs across 255k games.

### Pre-Training Checklist
Before starting any training run longer than 1 hour:
1. Verify `training_config.py` is not overridden by stale sweep configs
2. Check disk space (`df -h .` — need 10GB+ free for overnight runs)
3. Run smoke test (`uv run python scripts/v3_1h_test.py`) — confirm games complete, avg floor > 5
4. Confirm critical fixes are committed (action masking, value head, solver budgets)
5. Check no stale PIDs from previous runs (`bash scripts/training.sh status`)

### Config-Driven Scripts
- All training params come from `training_config.py` — NEVER hardcode values in scripts
- Scripts reference config via import, not duplicate values
- To change a param: edit config, not the script

### Disk Space
- Check before overnight runs. Auto-pause if < 5GB free.
- Archive old runs with timestamps (`logs/archive/YYYY-MM-DD_HH-MM/`), never `rm -rf`
- Checkpoints go to GitHub Releases (e.g., `v3-pretrain-checkpoints`), not git

### Session End
- Run `/reflect` at end of meaningful sessions.
- Updates memory, TODO.md, extracts rules.

## Training Pipeline (v3)
- **Config**: `training_config.py` — all params in one file
- **Pretrain**: BC warmup on trajectory data (43.7% accuracy baseline)
- **Loop**: COLLECT 100 games → TRAIN 10 PPO steps → SYNC weights → repeat
- **Combat**: TurnSolver (beam search, dynamic budget per enemy HP) + CombatNet (70/30 neural/heuristic)
- **Strategic**: Model picks cards, paths, rest, shop, events via policy head
- **Inference**: MLX on Metal GPU, 10 workers, batch timeout 15ms
- **Experiments**: Automated sweep via `v3_experiment_sweep.py` (BC+PPO winner at 8.11 avg floor)
- **Current best**: Experiment C (BC pretrain + PPO), 220 cycles, val_acc 55.9%, avg floor 8.9
- **Blocker**: Floor 16 wall (Act 1 boss). See `docs/research/floor16-root-cause.md`
- See `docs/TODO.md` for prioritized work items

## Key References
- `docs/vault/` — Game mechanics ground truth (damage, RNG, stances)
- `docs/research/` — RL methodology, training analysis, floor 16 analysis
- `docs/COMPLETED.md` — Full history of completed work with dates
- `CLAUDE-reference.md` — Engine API, parity tables, card data, RNG system details
- `~/Desktop/sts-archive/decompiled/` — Java source for verification
