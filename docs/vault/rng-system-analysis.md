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

- packages/engine-rs/src/seed.rs implements private native ports of libGDX
  `RandomXS128` and the 48-bit `java.util.Random` LCG. Public gameplay draws go
  through `StsRandom`, which owns the Java wrapper counter and exposes one
  named method per Java overload.
- packages/engine-rs/src/engine.rs owns combat streams and exports trace
  counters.
- packages/engine-rs/src/run.rs owns run generation and transition logic.
- packages/engine-rs/src/trace.rs defines the Rust trace schema.
- packages/engine-rs/src/bin/trace_replay.rs replays scripts against goldens.
- scripts/trace_diff.sh performs the offline comparison.
- data/traces/java/ contains the protected Java goldens.

Rust groups the seven run-persistent streams in `PersistentRngs` and the five
`seed + floorNum` streams in `FloorRngs`. Combat receives explicit snapshots of
the persistent `card` and `potion` streams plus all five floor streams, then
returns them through one `CombatRngs::absorb_into` operation. Map and Neow each
retain their independent streams. Raw generator transitions are private, and
production gameplay code does not use `rand`, `RngCore`, `SliceRandom`, or
generic sampling traits.

`Collections.shuffle` has two distinct Java paths and Rust preserves both:

- room assignment receives the wrapped `RandomXS128` directly, advancing raw
  state without incrementing the StS wrapper counter;
- card/relic shuffles first consume one signed `StsRandom.randomLong()`, then
  seed a separate native `java.util.Random` for Fisher-Yates.

The counted RNG implementation and ownership model are source-audited and
fixture-tested across all named streams. This proves the native algorithms,
counter contract, transfer/reset rules, and every currently covered call site;
it does not turn unexercised generated maps, pools, shops, events, rewards, or
encounters into full-run oracle evidence.

The two process-global generators are a separate certification boundary. Rust
implements their algorithms exactly and models deterministic semantic calls,
but the desktop game also consumes `MathUtils.random` from frame-, animation-,
audio-, and dialogue-dependent presentation code. A dungeon seed and action
script therefore cannot reconstruct the later desktop ambient cursor. Exact
desktop comparison requires the two-word MathUtils state and 48-bit default
Collections state at every settled checkpoint, or an equivalent ordered draw
witness. The headless simulator remains deterministic under its documented
initialization policy.
Use docs/work_units/audit-reports/engine-deep-audit.md and
docs/work_units/sim-completion-map.md for the active gaps; do not turn this
index into a hand-maintained parity table.

## Verification rule

For every RNG-sensitive change:

1. cite the exact Java call site;
2. assert both the selected outcome and counter delta in an engine-path test;
3. run the committed trace when one reaches the behavior; and
4. request a new human-minted golden when source evidence is insufficient.
