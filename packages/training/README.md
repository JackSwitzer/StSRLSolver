Combat-first training runtime built on the Rust engine contract.

Phase 1 is deliberately narrow:

- Watcher A0 combat only
- Rust-canonical combat observations and legal candidates
- snapshot-backed Rust PUCT collection + MLX policy/value learning
- append-only artifact logging
- frontier-preserving local action selection
- reconstructed Act 1 validation seed support
- MLX is the only supported backend

Core package surfaces:

- `contracts.py`
  - training-facing schemas and artifact payloads
- `bridge.py`
  - Rust/PyO3 loading helpers
- `stage2_pipeline.py`
  - canonical mixed snapshot corpus, Rust PUCT collection, and seed validation
- `inference_service.py`
  - policy/value batching, acting, and checkpoint updates
- `combat_model.py`
  - MLX policy/value model and checkpoint loader
- `benchmarking.py`
  - frontier scoring and benchmark grouping
- `selector.py`
  - temporary scalarized frontier acting rule used by phase-1 combat search
- `seed_suite.py`
  - external Watcher validation seeds

Canonical CLI:

```bash
./scripts/training.sh print-corpus-plan
./scripts/training.sh print-seed-suite
./scripts/training.sh launch --log-file logs/active/training-launcher.log --pid-file logs/active/training-launcher.pid run-phase1-puct-overnight --output-dir logs/active --target-cases 500 --collection-passes 3 --epochs 1
```

Artifact outputs:

- `manifest.json`: run manifest with git/config/backend truth
- `events.jsonl`: append-only lifecycle events
- `metrics.jsonl`: per-case collection metrics
- `system_stats.jsonl`: process/host CPU, RAM, swap, and best-effort GPU telemetry
- `frontier_report.json`: frontier rankings for the monitor
- `frontier_report.md`: human-readable frontier report
- `frontier_groups.json`: grouped frontier output
- `benchmark_report.json`: slice-level benchmark report
- `episodes.jsonl`: per-case replay/search summaries
- `puct_targets.jsonl`: canonical normalized root-visit policy/value targets
- `checkpoint.json`: MLX checkpoint snapshot
- `summary.json`: run summary with backend and validation status

Validation status meanings:

- `reconstructed_validated`: part of the required overnight gate
- `metadata_only`: reported but non-blocking

This package is the active training stack for PR #133.
