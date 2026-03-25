---
status: active
priority: P1
pr: null
title: Runtime Hardening — Monitoring, Exceptions, Recovery
scope: foundation
layer: runtime-hardening
created: 2026-03-25
completed: null
depends_on: [data-pipeline]
assignee: claude
tags: [testing, monitoring, reliability]
---

# Runtime Hardening

Improve reliability of the training pipeline through monitoring, exception handling, and auto-recovery.

## Disk Monitoring

- Auto-pause training if available disk drops below 5GB
- Alert (log + dashboard) at 10GB threshold
- Disk was at 3.5GB before emergency cleanup — this must not recur

## Exception Audit

- Log ALL caught exceptions to a dedicated file for review
- 3 silent `except Exception: pass` blocks caused critical bugs across 255k games
- Policy: no bare `except Exception: pass` — must log at WARNING+
- Allowed exceptions: `except queue.Empty`, `except KeyboardInterrupt`, documented expected exceptions
- Sweep codebase for remaining violations

## Config Verification Tests

- Runtime tests that run before training starts:
  - Solver budget matches config (boss fights must get 30s, not 20ms)
  - Action masking is enabled and correct
  - Config consistency between training_config.py and any script overrides
  - Model architecture matches checkpoint dimensions

## Boss Solver Runtime Test

- Verify boss fights actually receive the configured budget (was hardcoded to 20ms in scripts)
- Log actual solver time per fight tier (hallway, elite, boss)
- Alert if boss solver time < 5s (indicates misconfiguration)

## Action Mask Verification

- Test that invalid actions are never selected during collection
- Log any action mask violations as ERROR
- Include action mask stats in per-game metadata

## Auto-Recovery

- Restart crashed workers automatically (with backoff)
- Handle inference server failures (reconnect, fallback to CPU)
- Detect and recover from stale PID files
- Graceful degradation: continue training with fewer workers rather than full stop
