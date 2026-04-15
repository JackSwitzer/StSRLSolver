Combat-first training rebuild on top of the audited Rust engine.

Phase 1 is deliberately narrow:
- Watcher A0 combat only
- Rust-canonical combat observation and legal-candidate contracts
- curated + harvested corpus planning with deck provenance
- overnight frontier/search loop with append-only run artifacts
- frontier-preserving local selector for later meta-model handoff

Core runtime contract:
- `CombatTrainingState` carries schema versions, context, observation, and legal candidates.
- `RunManifest` records git/config provenance plus restriction policy and overnight-search settings.
- `EpisodeLog` / `BenchmarkReport` / `FrontierReport` are the monitored outputs for phase-1 promotion.

Corpus shape:
- starter deck baseline
- single-remove family for remove-heavy lines
- setup/upgrade family for opening-hand and potion variance
- opening-hand buckets and hard frontier cases harvested from overnight runs

Artifact model:
- manifest: `manifest.json`
- append-only events: `events.jsonl`
- append-only metrics: `metrics.jsonl`
- frontier summary: `frontier_report.json` + `frontier_report.md`
- frontier grouping: `frontier_groups.json`
- episode trace: `episodes.jsonl`

Implementation notes:
- The `packages/training_legacy` tree is historical reference only.
- This package is the active training stack for PR #133.
- Full details live in `docs/work_units/combat-first-training-rebuild.md`.
