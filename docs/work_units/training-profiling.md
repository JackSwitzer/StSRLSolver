---
status: active
priority: P1
pr: 133
title: Training Profiling
scope: training
layer: training-performance
created: 2026-04-15
completed: null
depends_on: [combat-first-training-rebuild]
assignee: claude
tags: [training, profiling, performance]
---

# Training Profiling

Profiling on this branch is about the new combat-first runtime, not a legacy collect/train loop.

## Main Questions

- how many searchable requests can we process overnight on the M4 before swap appears
- where is time going between Rust session work, inference, and model updates
- how stable are frontier and benchmark metrics across repeated runs

## Profiling Targets

### Rust Session / Snapshot Overhead
- combat snapshot export/import
- legal candidate export
- replay determinism checks

### Search / Policy-Value Throughput
- requests per second
- PUCT targets per second during updates
- frontier size and ranking stability

### Memory
- keep the default topology under roughly `20 GB` resident use
- avoid sustained swap during overnight runs

### Monitor-Friendly Output
- every run writes manifest, metrics, frontier, benchmark, checkpoint, and summary artifacts
- the app should remain readable while a run is actively writing

## Minimal Commands

```bash
./scripts/training.sh print-corpus-plan
./scripts/training.sh run-phase1-puct-overnight --output-dir logs/active --target-cases 24 --collection-passes 1 --epochs 1
```
