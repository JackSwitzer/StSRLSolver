---
status: active
priority: P0
pr: null
title: Data Pipeline — Consolidation, Quality, Loading
scope: foundation
layer: data-pipeline
created: 2026-03-25
completed: null
depends_on: []
assignee: claude
tags: [data, consolidation, quality, logging]
---

# Data Pipeline

Consolidate scattered training data, assess quality, and build shared infrastructure for loading and logging.

## Data Consolidation

- ~96k trajectory files and ~536k combat .npz files scattered across Desktop
- Inventory all data: locations, formats, sizes, date ranges
- Identify duplicates and corrupted files
- Organize into tiered structure:
  - `raw/` — unfiltered collection output
  - `filtered/` — dimension-consistent, valid transitions only
  - `curated/` — high-quality games (floor 20+, correct action masking)
  - `expert/` — Merl A20 seeds, known-good rollouts

## Quality Assessment

- Check dimension consistency across trajectory files (encoder output changed between versions)
- Verify action masking correctness (invalid actions should never appear as chosen)
- Flag games with anomalous reward signals (e.g. NaN, extreme outliers)
- Assess what fraction of existing data is actually usable for current architecture

## Shared Data Loading

- Extract shared data loading to `data_utils.py`
- Currently ~10 scripts with ~300 lines of duplicated loading logic
- Standardize: DataLoader, batching, shuffling, train/val splits
- Support streaming for large datasets that don't fit in memory

## Logging Spec

- Define logging format for future data collection (dashboard-ready)
- Per-decision metadata: state encoding, action probabilities, chosen action, reward
- Per-game metadata: seed, config hash, final floor, outcome
- See `docs/CLAUDE-data.md` for full format specs and existing TODO items
