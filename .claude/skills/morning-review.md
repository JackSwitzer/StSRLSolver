---
name: morning-review
description: Deep Opus-level analysis of overnight training data with parallel subagents
user-invocable: true
---

# Morning Training Review

Comprehensive analysis of overnight training run. Uses parallel subagents for maximum depth.

## Process

### Phase 1: Data Collection (3 parallel subagents)

**Agent 1 — Episode Deep Dive:**
- Read episodes.jsonl (full or sample last 10K)
- Floor distribution histogram
- Death analysis: floor x enemy matrix, top 20 killers
- Deck composition: high-floor vs low-floor card choices
- Combat stats: HP loss by encounter, turns per fight, zero-card combats
- Solver telemetry: solver_ms, solver_calls, multi-turn activations
- Duration analysis: sub-1s games (instant deaths), avg duration by floor
- Card pick patterns: what's being picked vs skipped
- Relic acquisition rate by floor
- Hand state analysis: playable cards unplayed, energy wasted, status card ratio

**Agent 2 — Training Metrics:**
- Parse nohup.log for ALL logged metrics over time
- Loss components: policy_loss, value_loss, entropy, aux_loss, clip_fraction
- Entropy trajectory + stall detector firings
- LR schedule anomalies
- Per-component loss: which term drives negative total?
- Phase timing: collect vs train duration, throughput degradation
- Buffer dynamics: transitions per collect, buffer utilization

**Agent 3 — Strategic Quality + Diagnostics:**
- Compare early vs late episodes (learning signal)
- Path selection patterns over time (evidence of learning?)
- Card pick distribution over time (evidence of card preference learning?)
- Combat logging: turns with energy remaining but no cards played
- Reward distribution: total_reward vs floor correlation
- PBRS behavior: does potential increase with floor or decrease?
- Value head accuracy: predicted value vs actual outcome
- Cross-reference with literature review if available

### Phase 2: Synthesis Report

Write to logs/weekend-run/morning_analysis.md:

1. Executive Summary (3 sentences)
2. Key Metrics Table (games, floor, win rate, loss, entropy, throughput)
3. Floor Distribution (ASCII histogram)
4. Learning Evidence (is the model learning? what specifically?)
5. Top 5 Issues (ranked by impact, with data evidence)
6. Reward System Health (is reward correlated with what we want?)
7. Combat System Health (solver working? hand/energy logging gaps?)
8. Recommended Changes (ranked by expected impact x ease)
9. Data Quality (is this data worth keeping for future training?)

### Phase 3: Present to User

Show executive summary and top recommendations. Offer to deep-dive on any section.
