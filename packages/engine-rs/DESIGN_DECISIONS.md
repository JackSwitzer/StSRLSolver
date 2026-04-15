# Design Decisions

Last updated: 2026-04-14  
Branch: `codex/universal-gameplay-runtime`

This file records durable, intentional runtime decisions so parity review can distinguish “we chose this” from “we missed this.”

## 1. Neow Always Exposes 4 Choices

Status: intentional RL-facing deviation

Policy:

- The run/action layer always surfaces `4` Neow options for a given seed.
- This intentionally overrides the vanilla Java progression gate that normally reduces the opening set for players who have not reached the Act 1 boss.

Why we do this:

- it gives bots the widest consistent start-of-run action surface
- it avoids training on an artificially narrowed opening distribution
- it keeps seed-conditioned opening decisions stable across evaluation/training contexts

Canonical surfaces:

- [run.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/run.rs:1)
- [decision.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/decision.rs:1)
- [test_run_parity.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_run_parity.rs:1)
- [test_rl_contract.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_rl_contract.rs:1)

## 2. `NoteForYourself` Cross-Run Card Persistence

Status: intentional runtime-supported future-run effect

Policy:

- `NoteForYourself` is the event matching the “crack in the wall / leave a card for future runs” description.
- The canonical runtime behavior is:
  - read the note
  - claim the stored card
  - choose one current deck card to save for a future run

Current implementation decision:

- the stored note card persists across runs inside the engine runtime process
- it defaults to `IronWave`
- it is surfaced through the canonical event reward + deck-selection flow rather than a side channel

Why this shape:

- it keeps the mechanic inside the same typed event/reward runtime as the rest of the engine
- it gives training/evaluation runs stable future-run behavior without depending on external profile-save plumbing

Canonical surfaces:

- [shrines.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/events/shrines.rs:1)
- [run.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/run.rs:1)
- [test_event_runtime_wave21.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_event_runtime_wave21.rs:1)

## 3. `Match and Keep!`

Status: intentionally not claimed complete yet

Policy:

- We do not treat the old temporary fixed-card approximation as merge-ready parity.
- Until the Java GremlinMatchGame runtime exists, `Match and Keep!` stays explicitly blocked in source and tests.

Why this choice:

- a fake fixed reward path hides a real gameplay-state gap
- the real event needs hidden tile state, reveal/match logic, and canonical event/reward integration

Canonical surfaces:

- [shrines.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/events/shrines.rs:1)
- [test_event_runtime_wave19.rs](/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine-rs/src/tests/test_event_runtime_wave19.rs:1)
- Java oracle: `/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src/com/megacrit/cardcrawl/events/shrines/GremlinMatchGame.java`
