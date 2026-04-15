# Training Architecture Reference

This branch uses the **combat-first training rebuild**.

## Current Goal

Phase 1 is a Watcher A0 combat solver with:

- Rust-canonical combat observations and legal candidates
- corpus-driven search and reanalysis training
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
- [packages/training/corpus.py](/Users/jackswitzer/Desktop/SlayTheSpireRL-training-rebuild/packages/training/corpus.py:1)
- [packages/training/inference_service.py](/Users/jackswitzer/Desktop/SlayTheSpireRL-training-rebuild/packages/training/inference_service.py:1)

## Overnight Run

Canonical bring-up command:

```bash
mkdir -p logs/active logs/runs
./scripts/training.sh run-phase1-overnight \
  --output-dir logs/active \
  --epochs 1 \
  --target-requests 24 \
  --backend linear
```

Useful planning/debug commands:

```bash
./scripts/training.sh print-default-config
./scripts/training.sh print-stack-config
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
- `dataset.jsonl`
- `checkpoint.json`
- `summary.json`

These are the supported monitoring and replay surfaces.

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

Phase 1 is mostly synthetic, but it already tracks deck provenance:

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
- manual replay/demo sessions in SpireMonitor
- future Baalorlord run import scaffolding

## What This Branch Is Not

- not the superseded training pipeline
- not a fixed 512-action head system
- not a backward-compatibility dashboard contract
- not a whole-run strategic learner yet

Strategic/pathing learning comes after the combat solver, corpus, and benchmark loop are stable.
