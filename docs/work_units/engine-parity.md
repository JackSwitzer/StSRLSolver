---
status: active
priority: P1
pr: null
title: Engine Parity — Events, Powers, Relics
scope: foundation
layer: engine-parity
created: 2026-03-25
completed: null
depends_on: []
assignee: claude
tags: [engine, parity, events, powers, relics]
---

# Engine Parity

Remaining parity work between the Python engine and the Java source. NOT blocking Watcher A20 training currently, but required for the 96% WR target.

## Events (biggest gap)

- 49 unchecked items (11% complete)
- This is the largest remaining parity gap by far
- Many events have branching outcomes that affect run strategy (e.g. Knowing Skull, Golden Idol, Wheel of Change)
- See `granular-events.md` for the full checklist
- **Priority**: Focus on Watcher-relevant events first (Act 1-3 event pools that appear in A20 runs)

## Powers

- 9 unchecked items (85% complete)
- All remaining items are Defect/Ironclad/enemy powers, NOT Watcher powers
- Not blocking Watcher training
- See `granular-powers.md` for the full checklist

## Relics

- 14 unchecked items (80% complete)
- Violet Lotus is implemented but untested
- Several remaining relics are class-specific for non-Watcher classes
- See `granular-relics.md` for the full checklist

## Approach

1. Audit Watcher event pool — identify which of the 49 unchecked events actually appear in Watcher runs
2. Implement Watcher-relevant events first (estimated ~20 of the 49)
3. Defer Defect/Ironclad-only powers and relics until multi-class support
4. Each event implementation needs: logic, tests, and Java source verification
