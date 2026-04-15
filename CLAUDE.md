# Slay the Spire RL Training Rebuild

This worktree is the stacked training branch, not the engine base branch.

## Active Branch Shape

- engine base branch: `codex/universal-gameplay-runtime`
- training branch: `codex/training-rebuild`
- current PR stack:
  - [PR #132](https://github.com/JackSwitzer/StSRLSolver/pull/132) -> `main`
  - [PR #133](https://github.com/JackSwitzer/StSRLSolver/pull/133) -> `codex/universal-gameplay-runtime`

## Active System

Use the new artifact-first training stack only:

- `packages/engine-rs/`
  - canonical Rust engine and training contract
- `packages/training/`
  - combat-first corpus, reanalysis loop, manifests, frontier logging
- `packages/app/SpireMonitor/`
  - manifest/frontier/benchmark/replay monitor

Do not reintroduce or document superseded training flows in this branch.

## Commands

```bash
./scripts/test_engine_rs.sh check --lib
./scripts/test_engine_rs.sh test --lib training_contract -- --nocapture
uv run pytest tests/training -q

./scripts/training.sh print-corpus-plan
./scripts/training.sh print-seed-suite
./scripts/training.sh launch --log-file logs/active/training-launcher.log --pid-file logs/active/training-launcher.pid run-phase1-puct-overnight --output-dir logs/active --target-cases 500 --collection-passes 3 --epochs 1 --backend mlx
```

## Key Docs

- `docs/CLAUDE-training.md`
- `docs/work_units/combat-first-training-rebuild.md`
- `docs/work_units/watcher-seed-validation-suite.md`

## Current Phase

- Watcher A0 combat first
- artifact-first monitor output
- synthetic-first corpus plus external validation seeds
- strategic/pathing training deferred until the combat loop is stable
