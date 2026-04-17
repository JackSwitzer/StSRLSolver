# Branch Reference

This file is the quick reference for the stacked training rebuild branch.

## Canonical Commands

```bash
./scripts/test_engine_rs.sh check --lib
./scripts/test_engine_rs.sh test --lib training_contract -- --nocapture
uv run pytest tests/training -q

./scripts/training.sh print-corpus-plan
./scripts/training.sh print-seed-suite
./scripts/training.sh launch --log-file logs/active/training-launcher.log --pid-file logs/active/training-launcher.pid run-phase1-puct-overnight --output-dir logs/active --target-cases 500 --collection-passes 3 --epochs 1
```

## Canonical Paths

- Rust contract:
  - `packages/engine-rs/src/training_contract.rs`
- training runtime:
  - `packages/training/`
- monitor app:
  - `packages/app/SpireMonitor/`
- active branch docs:
  - `docs/CLAUDE-training.md`
  - `docs/work_units/combat-first-training-rebuild.md`
  - `docs/work_units/watcher-seed-validation-suite.md`

## Active Artifacts

- `manifest.json`
- `events.jsonl`
- `metrics.jsonl`
- `frontier_report.json`
- `frontier_groups.json`
- `benchmark_report.json`
- `episodes.jsonl`
- `puct_targets.jsonl`
- `checkpoint.json`
- `summary.json`

## Current Scope

- Watcher A0 combat only
- snapshot-backed Rust PUCT collection + MLX policy/value learning
- reconstructed Act 1 validation seeds
- artifact-first monitoring
- MLX-only backend policy

Strategic/pathing learning comes later.
