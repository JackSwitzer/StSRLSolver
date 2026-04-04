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

## Rust parity suite snapshot (2026-04-01)

- The current Rust parity suite is the best source of truth for remaining engine-tail work.
- Verification baseline:
  - `PYO3_PYTHON=/Users/jackswitzer/Desktop/SlayTheSpireRL/.venv/bin/python3 cargo test --no-run`
  - `PYO3_PYTHON=/Users/jackswitzer/Desktop/SlayTheSpireRL/.venv/bin/python3 cargo test -- --format terse`
- Current state: 1686 total tests, 1634 passing, 52 intentionally failing runtime parity tests.
- Treat those failing tests as scoped work units, not as a signal to broaden the audit again.

## Current scoped Rust work units

- Boss ascension tables and move sequencing
  - 13 failing tests in `test_bosses.rs`
  - Examples: Guardian A2/A19 scaling, Hexaghost A4/A19 values, Collector opening/debuff flow, Donu/Deca/Heart ascension values
- Defect card and orb runtime parity
  - 11 failing tests in `test_cards_defect.rs`
  - Examples: Buffer install, Machine Learning draw status, Meteor Strike plasma channel, Storm trigger, Seek/Reboot tutor-draw flow
- Ironclad and Silent status/legality wiring
  - 12 failing tests across `test_cards_ironclad.rs` and `test_cards_silent.rs`
  - Examples: Weak/Vulnerable application, Shrug It Off draw, Clash legality, Grand Finale legality, Terror/Neutralize/Sucker Punch debuffs
- Watcher registry coverage
  - 7 failing tests in `test_cards_watcher.rs`
  - Missing registry entries/aliases: `Collect`, `DeusExMachina`, `Discipline`, `Wireheading`/Foresight, `Sanctity`, `Vengeance`/Simmering Fury, `Unraveling`
- Event catalog expansion
  - 3 failing tests in `test_events_parity.rs`
  - Rust dispatch still exposes only the partial event catalog instead of the Java act counts
- Power alias, trigger, and status-key cleanup
  - 6 failing tests in `test_powers_parity.rs`
  - Examples: Battle Hymn and Establishment metadata, After Image/Rage play triggers, Storm/Heatsink dispatch, Wave of the Hand status-key mismatch, Wraith Form type mismatch

## Notes from the audit

- Enemy AI breadth tests were tightened during the audit and are no longer part of the failing set.
- The remaining failures are now mostly useful parity gaps rather than brittle test expectations.
- When promoting parity work into implementation, prefer closing one failure bucket at a time and leave the other failing tests in place as documentation.

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
