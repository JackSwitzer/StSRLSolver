---
status: active
priority: P0
pr: 133
title: "Combat-First Training Rebuild — Phase 1 Contract"
scope: training
layer: training-architecture
created: 2026-04-15
completed: null
depends_on: [training-architecture, data-pipeline, runtime-hardening]
assignee: claude
tags: [training, combat-first, corpus, logging, frontier]
---

# Combat-First Training Rebuild

This doc captures the implemented phase-1 direction for the stacked training branch.
It is intentionally narrow: Watcher A0 combat first, with the rest of the stack
waiting behind it.

## Contract Surface

The training stack consumes the Rust engine through a typed combat contract:

- `CombatTrainingState` carries `schema_versions`, `context`, `observation`, and `legal_candidates`.
- `CombatObservation` is the canonical state snapshot for the combat-first model.
- `LegalActionCandidate` is the model-facing action set; training should never infer legality by decoding raw tags.
- `RestrictionPolicy` is explicit and config-driven; it belongs in the training layer, not the engine runtime.
- `RunManifest` records git/config provenance plus the overnight-search snapshot for reproducibility.
- `EpisodeLog` and `BenchmarkReport` are the two primary scored outputs for promotion.

The contract is Watcher A0 combat only for phase 1. Other game phases can exist in
the engine, but the benchmark corpus and overnight loop are combat-first.

## Corpus Families

The current corpus is organized around deck provenance, not just seed strings.
Each family carries `SeedProvenance`, `NeowProvenance`, and `DeckProvenance`.

Implemented families:

- `starter-vanilla`
  - pure Watcher starter deck
  - baseline hallway solves
- `single-strike-remove`
  - one Strike removed
  - remove-heavy family for slimmer opening hands and stronger early lines
- `calm-setup-upgrade`
  - upgraded Vigilance plus a light setup add
  - used for opening-hand enumeration and potion variance

The deck provenance model explicitly tracks:

- `base_deck`
- `removed_cards`
- `added_cards`
- `upgraded_cards`
- `potion_set`
- `tags`
- derived `remove_count`

Current case slices are grouped as:

- `curated-core`
  - baseline hallway and elite states
- `opening-hand-buckets`
  - enumerated opening-hand variance for the setup family
- `frontier-harvest-hard`
  - hard elite/boss cases harvested from search disagreement or weak solve rates

The current placeholder seed inventory has been replaced by an explicit external
Watcher validation suite, but the synthetic-first corpus still remains the
primary training source.

## Overnight Phase-1 Loop

The overnight loop is a collect -> search -> log -> evaluate cycle:

1. Build a `TrainingRunManifest` from the current git/config snapshot and restriction policy.
2. Load the Watcher A0 corpus plan and the overnight search settings.
3. Run combat collection with the local selector/search policy.
4. Emit append-only artifacts for each run.
5. Aggregate frontier results by corpus axes.
6. Promote only if the benchmark improves without violating the runtime budget.

The configured overnight search snapshot includes:

- `sweep_config`
- `search_policy`
- `planned_games`
- `worker_count`
- `corpus_name`
- `corpus_slices`
- `benchmark_groups`
- `easy_seed_bucket`
- `easy_seed_target_count`
- `neow_policy`
- `budget`

Promotion gates should be based on:

- solve rate
- expected HP loss
- expected turns
- oracle top-k agreement
- p95 elapsed time
- p95 RSS

## Monitor Artifact Model

The runtime artifact tree is append-only and intentionally simple:

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

What each file is for:

- `manifest.json`
  - reproducibility and provenance
- `events.jsonl`
  - phase changes and run-level state transitions
- `metrics.jsonl`
  - step metrics with corpus provenance fields
- `frontier_report.json` / `.md`
  - frontier ranking, best-by-metric, and group summaries
- `frontier_groups.json`
  - machine-readable corpus grouping for dashboards
- `benchmark_report.json`
  - benchmark slice results and promotion gates
- `episodes.jsonl`
  - episode traces with corpus slice/case provenance
- `dataset.jsonl`
  - reanalysis examples captured for model updates
- `checkpoint.json`
  - lightweight phase-1 model weights
- `summary.json`
  - single-run completion summary

Episode provenance should carry:

- `corpus_slice`
- `corpus_case`
- `deck_family`
- `remove_count`
- `potion_set`
- `enemy`
- `seed_source`
- `neow_source`

## Canonical Run Flow

Default overnight smoke / bring-up command:

```bash
mkdir -p logs/active logs/runs
./scripts/training.sh run-phase1-overnight \
  --output-dir logs/active \
  --epochs 1 \
  --target-requests 24 \
  --backend linear
```

Monitor flow:

```bash
cd packages/app
swift build
open SpireMonitor
```

The app is configured to read `logs/active` via `.spire-monitor.json`.

## What This Phase Is Not

- Not a whole-run A20 training benchmark.
- Not a generic engine parity exercise.
- Not a cross-repo Python rewrite.
- Not a legacy stack continuation.
