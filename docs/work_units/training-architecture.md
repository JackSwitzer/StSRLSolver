---
status: active
priority: P0
pr: 133
title: Combat-First Training Architecture
scope: training
layer: training-architecture
created: 2026-04-15
completed: null
depends_on: [combat-first-training-rebuild]
assignee: claude
tags: [training, combat-first, architecture]
---

# Training Architecture

The current branch architecture is:

- Rust engine legality and state are canonical
- Python orchestrates snapshot corpus generation, Rust-PUCT target collection, policy/value updates, and artifact logging
- MLX-backed model evaluation/training is the intended GPU path
- SpireMonitor consumes the new artifact set directly

## Phase 1

Scope:

- Watcher A0 combat only
- legal-candidate scoring, not a fixed global action head
- multi-objective combat outcome vector
- frontier-preserving local selector
- synthetic-first corpus plus external validation seeds

What phase 1 must prove:

- we can run overnight training jobs repeatedly
- weights update from Rust-PUCT policy/value targets
- benchmark slices are stable and comparable
- frontier/replay artifacts are visible in the app

## Phase 2 Preparation

Phase 1 data should preserve:

- deck provenance
- remove count
- Neow provenance
- potion set
- opening-hand bucket
- benchmark group identity

That allows a later strategic learner to reason about:

- removes vs adds
- potion spend vs HP preservation
- route and Neow value

## Not In Scope Yet

- full-run strategic training
- whole-run win-rate optimization
- a second high-level model
- legacy compatibility layers
