---
status: active
priority: P1
pr: null
title: Dashboard — Decision Quality, Training Curves, Data Inventory
scope: visibility
layer: dashboard
created: 2026-03-25
completed: null
depends_on: [data-pipeline]
assignee: claude
tags: [app, swiftui, visualization]
---

# Dashboard

Improvements to the SwiftUI macOS monitoring dashboard for training visibility.

## Current State

The dashboard already has:
- DiagnosticsCharts view
- SweepComparison view
- CardPickSummary view
- Episode logging integration (PR #62)

## Decision Quality

- Per-decision breakdown: chosen action vs optimal action, model confidence
- Card pick analysis: offered cards vs chosen, model probability distribution
- Path choice visualization: which paths the model prefers and why
- Event decision tracking: choices made at each event node

## Training Convergence

- Live loss curves: policy loss, value loss, entropy, total loss
- Clip fraction over time (PPO health indicator)
- Value accuracy: predicted vs actual returns
- Anomaly detection: alert on loss spikes, clip fraction > 0.3, value divergence
- Learning rate schedule visualization

## Data Inventory

- How much data exists per tier (raw, filtered, curated, expert)
- Quality distribution: histogram of game lengths, win rates by config
- Data freshness: when was each tier last updated
- Disk usage breakdown by data category

## Run Comparison

- Side-by-side config comparison on same seeds
- Overlay training curves from different experiments
- Statistical significance testing (are differences real or noise)
- Export comparison reports

## UI Improvements

- Cleaner drill-in menus for per-game inspection
- Better controls: date range filters, config selectors, search
- Keyboard shortcuts for common operations
- Dark mode refinements
