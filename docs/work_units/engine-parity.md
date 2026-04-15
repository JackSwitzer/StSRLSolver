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

Legacy parity work between the Python engine and the Java source. This is not blocking current Watcher training, but it still matters for the long-run WR target.

## Events

- The granular event checklist is a legacy reference now, not a trustworthy open-gap count.
- Use `granular-events.md` only to find the remaining event-tail items that still need explicit follow-up.
- Prioritize Watcher-relevant event behavior first when adding parity work.

## Powers

- The granular power checklist is also a legacy reference, with most remaining items outside the Watcher core.
- Use `granular-powers.md` only for the residual power-tail items that still need verification or cleanup.

## Relics

- The granular relic checklist is likewise a legacy reference.
- Use `granular-relics.md` for the remaining relic-tail items and any doc/runtime mismatches that still need a source-of-truth decision.

## Approach

1. Treat the granular files as follow-up references, not live completion counters.
2. Keep the remaining parity follow-up small and targeted.
3. Add logic, tests, and Java-source verification together when a parity tail item is promoted into implementation.
