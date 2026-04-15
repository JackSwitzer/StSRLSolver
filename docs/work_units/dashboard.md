---
status: active
priority: P1
pr: 133
title: SpireMonitor Artifact Views
scope: visibility
layer: dashboard
created: 2026-04-15
completed: null
depends_on: [combat-first-training-rebuild]
assignee: claude
tags: [app, swiftui, visualization]
---

# SpireMonitor Artifact Views

The monitor on this branch is artifact-first.

## Current Surfaces

- active run summary
- benchmark slice dashboard
- frontier inspector
- event stream
- metric stream
- system stats

## Required Inputs

- `manifest.json`
- `events.jsonl`
- `metrics.jsonl`
- `frontier_report.json`
- `benchmark_report.json`
- `episodes.jsonl`
- `summary.json`

## What The App Should Make Easy

- compare chosen frontier lines to alternatives
- inspect replayable frontier-bearing episode steps
- slice benchmark performance by deck family and enemy
- verify a run is active and writing artifacts to `logs/active`

## Near-Term Polish

- better replay detail for frontier-bearing steps
- clearer benchmark grouping labels
- clearer run provenance presentation from the manifest
