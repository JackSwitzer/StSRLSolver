# RNG source index

This file is an index, not a parity claim. The decompiled game defines RNG
semantics; committed Java traces are the behavioral oracle. Rust code and tests
remain implementations to check against those sources.

## Canonical trace keys

TraceLab emits these 13 counters from
packages/harness-java/src/main/java/tracelab/TraceWriter.java:

- card
- cardRandom
- shuffle
- monster
- monsterHp
- ai
- relic
- treasure
- event
- merchant
- potion
- map
- misc

The names above are the trace-schema vocabulary. Do not merge card and
cardRandom, or monster and monsterHp.

## Java authority

- decompiled/java-src/com/megacrit/cardcrawl/random/Random.java defines the
  generator and counter behavior.
- decompiled/java-src/com/megacrit/cardcrawl/dungeons/AbstractDungeon.java
  declares the dungeon streams, initializes them from Settings.seed, restores
  persistent counters from save data, and performs room-transition reseeding.
- AbstractDungeon.nextRoomTransition reseeds monsterHp, ai, shuffle,
  cardRandom, and misc with Settings.seed + floorNum. Other stream lifecycles
  must be derived from their call sites rather than inferred from this list.
- decompiled/java-src/com/megacrit/cardcrawl/helpers/SeedHelper.java defines
  displayed-seed conversion.

Search the full Java source for the exact stream at every random decision. A
secondary randomBoolean, randomFloat, retry roll, singleton selection, or sound
selection can be parity-significant because it advances the stream counter.

## Local implementation and oracle

- packages/engine-rs/src/seed.rs implements the Java-style generator used by
  Rust.
- packages/engine-rs/src/engine.rs owns combat streams and exports trace
  counters.
- packages/engine-rs/src/run.rs owns run generation and transition logic.
- packages/engine-rs/src/trace.rs defines the Rust trace schema.
- packages/engine-rs/src/bin/trace_replay.rs replays scripts against goldens.
- scripts/trace_diff.sh performs the offline comparison.
- data/traces/java/ contains the protected Java goldens.

The current engine does not yet prove all 13 run-level streams independently.
Use docs/work_units/audit-reports/engine-deep-audit.md and
docs/work_units/sim-completion-map.md for the active gaps; do not turn this
index into a hand-maintained parity table.

## Verification rule

For every RNG-sensitive change:

1. cite the exact Java call site;
2. assert both the selected outcome and counter delta in an engine-path test;
3. run the committed trace when one reaches the behavior; and
4. request a new human-minted golden when source evidence is insufficient.
