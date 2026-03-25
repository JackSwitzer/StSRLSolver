# Codex Task: Performance Optimization

See PR description for full instructions. Key areas:

- GPU utilization: currently 18%, target >50%
- CPU bottleneck: Python game engine is the limiter
- Rust engine: `origin/feat/rust-engine-expansion` has 516 passing tests
- Concurrent training: `scripts/v3_train_concurrent.py` — train thread + collect thread
- Batch sizes: model only uses 300MB of 24GB available
- MCTS budget allocation: `packages/training/training_config.py` SOLVER_BUDGETS

Use subagents: one for profiling/analysis, one for optimization recommendations.
See `docs/codex-optimization-audit.md` for the full detailed prompt.
