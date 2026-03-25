---
status: reference
priority: P2
pr: null
title: "Combat Logging & Replay"
scope: foundation
layer: engine-parity
created: 2026-02-23
completed: null
depends_on: []
assignee: claude
tags: [engine, combat, logging, replay]
---

# Work Unit: Combat Logging & Replay

## Goal
Enhanced turn-level combat logging for diagnosing boss fight failures and building a replay viewer.

## Current State
Turn-level logging already captures (added session 12):
- Cards played per turn
- Hand at end of turn
- Energy remaining
- Player HP/block/stance
- Enemy HP/block/intent
- Playable cards not played

## New Additions

### Boss HP Threshold Logging
- Log when boss HP crosses 75%, 50%, 25% thresholds
- Include turn number and player state at each threshold
- Feed into reward shaping (see reward-tuning.md)

### Boss Fight Replay Data
- Full turn-by-turn state for all boss fights
- Deck state (draw/discard/exhaust pile sizes)
- Power/buff stacks on player and enemies
- Relic trigger events

### App Replay View
- Visual combat replay from enhanced logs
- Step through turns forward/backward
- Show card plays, damage numbers, stance changes
- Color-code turns by quality (wrath damage, block efficiency)

### Data Pipeline
- `worker.py`: Already captures turns_detail per combat
- `enrich_episodes.py`: Add boss threshold analysis
- `top_episodes.json`: Include full replay data for top 200 episodes
- App: New ReplayView.swift component

## Dependencies
- Turn-level logging (already implemented)
- top_episodes.json enrichment pipeline

## Metrics
- Boss fight diagnosis: can we identify the turn where the game was lost?
- Turn quality heuristic: cards played / playable cards ratio
