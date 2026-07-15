# Enemy AI source index

This file replaces the old hand-maintained move tables. Decompiled Java and
committed action traces are authoritative; the Rust implementation and tests
must be checked against them.

## Shared Java contract

- decompiled/java-src/com/megacrit/cardcrawl/monsters/AbstractMonster.java
  defines rollMove, setMove, move history, intent construction, and shared
  turn behavior.
- reference/extracted/methods/base/AbstractMonster.java is the compact extract
  used for routine audit work.
- AbstractMonster.rollMove consumes AbstractDungeon.aiRng.random(99) before
  dispatching to the monster's getMove. A monster may consume additional aiRng
  values inside getMove or takeTurn; those draws must be preserved exactly.
- lastMove and lastTwoMoves constraints are part of the state machine, not
  optional anti-repeat heuristics.

## Per-monster authority

- Full sources: decompiled/java-src/com/megacrit/cardcrawl/monsters/
- Extracts: reference/extracted/methods/monster/

Read the concrete constructor, getMove, takeTurn, and damage table for the
monster being audited. Ascension branches, multi-enemy coordination, spawn
behavior, sound/dialogue RNG, and recursive rerolls are all per-class concerns
and must not be inferred from another monster.

## Rust touchpoints

- packages/engine-rs/src/enemies/mod.rs owns enemy construction and shared
  initial/next-move dispatch.
- packages/engine-rs/src/enemies/act1.rs through act4.rs contain per-act move
  logic.
- packages/engine-rs/src/engine.rs owns combat turn execution and ai_rng.
- packages/engine-rs/src/run.rs selects and enters encounters.
- packages/engine-rs/src/tests/test_enemies.rs and adjacent source-cited tests
  cover engine behavior.

The source-verification ledger covers 68 monster content rows, but that does not
prove encounter selection, run-level seeding, spawn order, or full trace parity.
The committed trace corpus currently exercises only a short Jaw Worm sequence.
See docs/work_units/audit-reports/engine-deep-audit.md and
docs/work_units/sim-completion-map.md for open system work.

## Required test shape

For each AI correction, drive the production initial-move or next-move path and
assert:

1. the selected move and intent fields;
2. relevant move history;
3. state changes from takeTurn;
4. exact aiRng counter deltas, including conditional secondary draws; and
5. ascension-dependent branches at their threshold.

When Java source leaves an ordering question unresolved, request a focused
TraceLab script rather than adding a prediction table here.
