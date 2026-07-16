# Ascension source index

This file is a routing guide, not an A1–A20 value table. Ascension behavior is
distributed across the decompiled game, and old summary tables can silently mix
base values, ascension thresholds, and display-adjusted values.

## Authority

Search for AbstractDungeon.ascensionLevel in the relevant Java subsystem:

- decompiled/java-src/com/megacrit/cardcrawl/characters/ for starting player
  state and character-specific setup;
- decompiled/java-src/com/megacrit/cardcrawl/monsters/ for HP, damage, powers,
  and AI threshold branches;
- decompiled/java-src/com/megacrit/cardcrawl/events/ for worse event outcomes;
- decompiled/java-src/com/megacrit/cardcrawl/rooms/ for room and reward rules;
- decompiled/java-src/com/megacrit/cardcrawl/shop/ for shop pricing behavior;
- decompiled/java-src/com/megacrit/cardcrawl/dungeons/ for act transitions,
  encounter pools, and global run flow;
- decompiled/java-src/com/megacrit/cardcrawl/cards/ and rewards/ for card reward
  behavior.

Use reference/extracted/methods/ for content items when an extract exists, then
open the full Java class for constructor and base-class context.

## Rust touchpoints

- packages/engine-rs/src/run.rs owns run setup, room entry, encounter
  construction, rewards, shops, and Neow flow.
- packages/engine-rs/src/enemies/ contains monster thresholds and move logic.
- packages/engine-rs/src/events/ contains event choices and outcomes.

Do not treat the 667-row content ledger as proof of the surrounding A20 run
rules. The committed Java trace corpus is currently A0 only, so it provides no
golden evidence for A1–A20 behavior.

## Verification rule

For each ascension-sensitive behavior:

1. cite the exact Java threshold and value source;
2. test the boundary immediately below and at the threshold;
3. route spawn-time adjustments through the production run entry path;
4. assert RNG consumption when the ascension branch changes a random choice;
5. add an A20 golden only through a human TraceLab minting session.

Current coverage and missing layers belong in
docs/work_units/sim-completion-map.md, not in a duplicated modifier table.
