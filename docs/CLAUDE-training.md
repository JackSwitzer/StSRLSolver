# Training Architecture Reference

This branch uses the **combat-first training rebuild**.

## Current Goal

Phase 1 is a Watcher A0 combat solver with:

- Rust-canonical combat observations and legal candidates
- snapshot-backed Rust PUCT collection and MLX policy/value training
- frontier-preserving action selection
- append-only manifests, metrics, benchmark, and replay artifacts
- SpireMonitor support for the new artifact model

This is the current supported path for [PR #133](https://github.com/JackSwitzer/StSRLSolver/pull/133).

## Active Contract

The training runtime consumes the Rust engine through a typed contract:

- `CombatTrainingState`
- `CombatObservation`
- `LegalActionCandidate`
- `CombatOutcomeVector`
- `CombatFrontierSummary`
- `RestrictionPolicy`
- `RunManifest`
- `EpisodeLog`
- `BenchmarkReport`

The canonical Rust surface starts in:

- [packages/engine-rs/src/training_contract.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL-training-rebuild/packages/engine-rs/src/training_contract.rs:1)

The Python bridge and runtime live in:

- [packages/training/bridge.py](/Users/jackswitzer/Desktop/SlayTheSpireRL-training-rebuild/packages/training/bridge.py:1)
- [packages/training/cli.py](/Users/jackswitzer/Desktop/SlayTheSpireRL-training-rebuild/packages/training/cli.py:1)
- [packages/training/stage2_pipeline.py](/Users/jackswitzer/Desktop/SlayTheSpireRL-training-rebuild/packages/training/stage2_pipeline.py:1)
- [packages/training/inference_service.py](/Users/jackswitzer/Desktop/SlayTheSpireRL-training-rebuild/packages/training/inference_service.py:1)

## Overnight Run

Canonical bring-up command:

```bash
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

Foreground shakedown command:

```bash
./scripts/training.sh run-phase1-puct-overnight \
  --output-dir logs/active \
  --target-cases 24 \
  --collection-passes 1 \
  --epochs 1
```

Useful planning/debug commands:

```bash
./scripts/training.sh print-corpus-plan
./scripts/training.sh print-seed-suite
```

## Artifact Model

The current training stack writes:

- `manifest.json`
- `events.jsonl`
- `metrics.jsonl`
- `frontier_report.json`
- `frontier_report.md`
- `frontier_groups.json`
- `benchmark_report.json`
- `episodes.jsonl`
- `puct_targets.jsonl`
- `checkpoint.json`
- `summary.json`

These are the supported monitoring and replay surfaces.

Artifact meanings:

- `manifest.json`: run identity, git/config snapshot, and backend truth
- `events.jsonl`: append-only lifecycle events
- `metrics.jsonl`: per-case collection metrics
- `frontier_report.json` / `frontier_report.md`: aggregate frontier summaries
- `frontier_groups.json`: monitor-ready grouped frontier slices
- `benchmark_report.json`: benchmark slice rollups
- `episodes.jsonl`: replay/search summaries for each collected case
- `puct_targets.jsonl`: normalized policy/value targets emitted from Rust PUCT
- `checkpoint.json`: MLX checkpoint snapshot
- `summary.json`: run-level summary for monitor and audit

## Monitor

SpireMonitor is artifact-first on this branch.

Supported views:

- active run summary
- benchmark slice dashboard
- frontier inspector
- event and metric streams
- system stats

Build/run:

```bash
cd packages/app
swift build
open SpireMonitor
```

The app reads `.spire-monitor.json`, which already points to `logs/active`.

## Corpus and Seed Validation

Phase 1 uses a mixed snapshot corpus and tracks deck provenance:

- deck list and upgrades
- remove count and removed-card history when known
- potion set
- Neow provenance
- opening-hand bucket metadata

The external Watcher validation bank lives in:

- [packages/training/seed_suite.py](/Users/jackswitzer/Desktop/SlayTheSpireRL-training-rebuild/packages/training/seed_suite.py:1)

The seed suite is for:

- easy/high-roll validation
- remove-heavy and minimalist-style checks
- reconstructed Act 1 replay/demo sessions in SpireMonitor
- future Baalorlord run import scaffolding

Overnight gate semantics:

- `4AWM3ECVQDEWJ` and `4VM6JKC3KR3TD` are the required reconstructed seeds
- `1TPMUARFP690B` is metadata-only and reported as non-blocking

## What This Branch Is Not

- not the superseded training pipeline
- not a fixed 512-action head system
- not a backward-compatibility dashboard contract
- not a whole-run strategic learner yet

Strategic/pathing learning comes after the combat solver, corpus, and benchmark loop are stable.
