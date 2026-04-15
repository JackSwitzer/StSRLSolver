Combat-first training runtime built on the Rust engine contract.

Phase 1 is deliberately narrow:

- Watcher A0 combat only
- Rust-canonical combat observations and legal candidates
- corpus-driven search + reanalysis training
- append-only artifact logging
- frontier-preserving local action selection
- external validation seed support

Core package surfaces:

- `contracts.py`
  - training-facing schemas and artifact payloads
- `bridge.py`
  - Rust/PyO3 loading helpers
- `corpus.py`
  - Watcher A0 corpus planning with deck provenance
- `inference_service.py`
  - reanalysis loop and model-service wiring
- `combat_model.py`
  - lightweight linear + MLX model backends
- `benchmarking.py`
  - frontier scoring and benchmark grouping
- `seed_suite.py`
  - external Watcher validation seeds

Canonical CLI:

```bash
./scripts/training.sh print-corpus-plan
./scripts/training.sh print-seed-suite
./scripts/training.sh launch --log-file logs/active/training-launcher.log --pid-file logs/active/training-launcher.pid run-phase1-puct-overnight --output-dir logs/active --target-cases 500 --collection-passes 3 --epochs 1 --backend mlx
```

Artifact outputs:

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

This package is the active training stack for PR #133.
