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
  inference_server.py  # MLX batch inference on Metal GPU
  state_encoders.py    # RunStateEncoder (480-dim) + CombatStateEncoder
  turn_solver.py       # Beam search combat solver (dynamic budgets)
  reward_config.py     # Thin adapter over training_config (hot-reloadable)
  sweep_config.py      # Experiment configs (references training_config)
  seed_pool.py         # Seed rotation + 12 Merl A20 expert seeds
  mlx_inference.py     # MLX model (mirrors PyTorch for Apple Silicon)
packages/app/          # SwiftUI macOS monitoring dashboard
packages/server/       # WebSocket server for dashboard
tests/                 # 6076+ tests (pytest)
scripts/               # training.sh, services.sh, hotfix.sh, app.sh
docs/                  # vault/ (game mechanics), research/, TODO.md
```

## Commands
```bash
uv run pytest tests/ -q                    # All tests
bash scripts/training.sh start --games N   # Start training
bash scripts/training.sh status            # Live metrics
bash scripts/training.sh stop              # Graceful shutdown
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

### Session End
- Run `/reflect` at end of meaningful sessions.
- Updates memory, TODO.md, extracts rules.

## Training Pipeline (high-level)
- **Config**: `training_config.py` — all params in one file
- **Loop**: COLLECT 100 games → TRAIN 10 PPO steps → SYNC weights → repeat
- **Combat**: TurnSolver (beam search, dynamic budget per enemy HP)
- **Strategic**: Model picks cards, paths, rest, shop, events via policy head
- **Inference**: MLX on Metal GPU, 10 workers, batch timeout 15ms
- See `docs/research/weekend-training-plan-2026-03-19.md` for current strategy
- See `docs/TODO.md` for prioritized work items

## Key References
- `docs/vault/` — Game mechanics ground truth (damage, RNG, stances)
- `docs/research/` — RL methodology, training analysis, weekend plans
- `CLAUDE-reference.md` — Engine API, parity tables, card data, RNG system details
- `~/Desktop/sts-archive/decompiled/` — Java source for verification
