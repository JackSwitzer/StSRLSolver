# Slay the Spire RL

Combat-first training rebuild stacked on top of the audited Rust engine base.

This branch is the training worktree and stacked draft PR:

- engine base: `codex/universal-gameplay-runtime` via [PR #132](https://github.com/JackSwitzer/StSRLSolver/pull/132)
- training rebuild: `codex/training-rebuild` via [PR #133](https://github.com/JackSwitzer/StSRLSolver/pull/133)

Current scope is intentionally narrow:

- Watcher A0 combat only
- Rust-canonical combat observation and legal-candidate contracts
- corpus-driven search + reanalysis training
- artifact-first monitoring in SpireMonitor
- external validation seeds for easy/minimalist-style Watcher checks

## Active Components

| Component | Location | Role |
|-----------|----------|------|
| Rust engine base | `packages/engine-rs/` | Canonical legality, combat state, training contract, snapshots |
| Training runtime | `packages/training/` | Corpus planning, reanalysis loop, manifests, artifact logging |
| SpireMonitor | `packages/app/SpireMonitor/` | SwiftUI monitor for manifests, frontiers, benchmarks, and replay traces |
| Training entrypoint | `scripts/training.sh` | Canonical CLI wrapper |

## Quick Start

```bash
uv sync

./scripts/test_engine_rs.sh check --lib
./scripts/test_engine_rs.sh test --lib training_contract -- --nocapture
uv run pytest tests/training -q

./scripts/training.sh print-corpus-plan
./scripts/training.sh print-seed-suite

mkdir -p logs/active logs/runs
./scripts/training.sh run-phase1-overnight \
  --output-dir logs/active \
  --epochs 1 \
  --target-requests 24 \
  --backend linear
```

Monitor:

```bash
cd packages/app
swift build
open SpireMonitor
```

The app reads `.spire-monitor.json`, which is already configured to look at `logs/active`.

## Artifact Model

The active training stack writes artifact-first outputs:

- `manifest.json`
- `events.jsonl`
- `metrics.jsonl`
- `frontier_report.json`
- `frontier_report.md`
- `frontier_groups.json`
- `benchmark_report.json`
- `episodes.jsonl`
- `dataset.jsonl`
- `checkpoint.json`
- `summary.json`

These are the supported monitor and audit surfaces for this branch.

## External Validation Seeds

The machine-readable Watcher validation suite lives in:

- [packages/training/seed_suite.py](/Users/jackswitzer/Desktop/SlayTheSpireRL-training-rebuild/packages/training/seed_suite.py:1)

The current suite emphasizes:

- easy/high-roll starts
- remove-heavy and minimalist-style lines
- Neow transforms and rare-card opens
- a couple negative-control seeds

## Branch Notes

- This branch does not preserve superseded training paths.
- Engine legality remains canonical in Rust.
- Restrictions such as `NoCardAdds` and `UpgradeRemoveOnly` belong in the training layer above legality.
- Strategic/pathing learning is deferred until the combat solver, corpus, and benchmark loop are stable.
