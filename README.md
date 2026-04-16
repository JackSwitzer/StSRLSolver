# Slay the Spire RL

Combat-first training rebuild stacked on top of the audited Rust engine base.

This branch is the training worktree and stacked draft PR:

- engine base: `codex/universal-gameplay-runtime` via [PR #132](https://github.com/JackSwitzer/StSRLSolver/pull/132)
- training rebuild: `codex/training-rebuild` via [PR #133](https://github.com/JackSwitzer/StSRLSolver/pull/133)

Current scope is intentionally narrow:

- Watcher A0 combat only
- Rust-canonical combat observation and legal-candidate contracts
- snapshot-backed Rust PUCT collection and MLX policy/value learning
- artifact-first monitoring in SpireMonitor
- reconstructed Act 1 validation seeds for easy/minimalist-style Watcher checks
- MLX is the only supported backend on this branch

## Active Components

| Component | Location | Role |
|-----------|----------|------|
| Rust engine base | `packages/engine-rs/` | Canonical legality, combat state, training contract, snapshots |
| Training runtime | `packages/training/` | Snapshot corpus generation, Rust PUCT collection, policy/value learning, manifests, artifact logging |
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
./scripts/training.sh launch \
  --log-file logs/active/training-launcher.log \
  --pid-file logs/active/training-launcher.pid \
  run-phase1-puct-overnight \
  --output-dir logs/active \
  --target-cases 500 \
  --collection-passes 3 \
  --epochs 1
```

Monitor:

```bash
cd packages/app
swift build
open SpireMonitor
```

The app reads `.spire-monitor.json`, which is already configured to look at `logs/active`.

For attached foreground debugging, use:

```bash
./scripts/training.sh run-phase1-puct-overnight --output-dir logs/active --target-cases 24 --collection-passes 1 --epochs 1
```

## Artifact Model

The active training stack writes artifact-first outputs:

- `manifest.json`: run identity, git snapshot, config snapshot, and backend policy truth
- `events.jsonl`: append-only lifecycle events for corpus generation, collection, training, and validation
- `metrics.jsonl`: per-case collection metrics including root visits and solve probability
- `system_stats.jsonl`: process/host CPU, RAM, swap, and best-effort GPU telemetry during the run
- `frontier_report.json`: aggregate frontier ranking output for monitor consumption
- `frontier_report.md`: human-readable frontier summary
- `frontier_groups.json`: grouped frontier slices for the app
- `benchmark_report.json`: benchmark slice rollups from the collected corpus
- `episodes.jsonl`: per-case replay/search summaries including frontier and value payloads
- `puct_targets.jsonl`: canonical normalized root-visit policy/value targets used for learning
- `checkpoint.json`: MLX checkpoint snapshot
- `summary.json`: run-level summary including backend loaded truth and seed-validation status

These are the supported monitor and audit surfaces for this branch.

Artifact log-shape reference for Claude and SpireMonitor:

- [docs/training-log-shape.md](/Users/jackswitzer/Desktop/SlayTheSpireRL-training-rebuild/docs/training-log-shape.md:1)

## External Validation Seeds

The machine-readable Watcher validation suite lives in:

- [packages/training/seed_suite.py](/Users/jackswitzer/Desktop/SlayTheSpireRL-training-rebuild/packages/training/seed_suite.py:1)

The current suite emphasizes:

- easy/high-roll starts
- remove-heavy and minimalist-style lines
- Neow transforms and rare-card opens
- two reconstructed overnight-gate seeds: `4AWM3ECVQDEWJ` and `4VM6JKC3KR3TD`
- one metadata-only non-blocking seed: `1TPMUARFP690B`

## Branch Notes

- This branch does not preserve superseded training paths.
- Engine legality remains canonical in Rust.
- The temporary scalarized frontier acting rule is implemented in `packages/training/selector.py` and documented as a phase-1 combat-only helper, not a second backend or alternate pipeline.
- Restrictions such as `NoCardAdds` and `UpgradeRemoveOnly` belong in the training layer above legality.
- Strategic/pathing learning is deferred until the combat solver, corpus, and benchmark loop are stable.
