# Design Decisions

Last updated: 2026-07-20
Branch: `codex/oracle-loop-wave3`

This file records durable, intentional runtime decisions so parity review can distinguish “we chose this” from “we missed this.”

## 1. Neow Always Exposes 4 Choices

Status: intentional action-surface deviation

Policy:

- The run/action layer always surfaces `4` Neow options for a given seed.
- This intentionally overrides the vanilla Java progression gate that normally reduces the opening set for players who have not reached the Act 1 boss.

Why we do this:

- it gives every consumer the widest consistent start-of-run action surface
- it avoids coupling simulation legality to external profile progression
- it keeps seed-conditioned opening decisions stable across evaluation/training contexts

Canonical surfaces:

- [`src/run.rs`](src/run.rs)
- [`src/decision.rs`](src/decision.rs)
- [`src/tests/test_run_parity.rs`](src/tests/test_run_parity.rs)
- [`../docs/work_units/parity-deviations-register.md`](../../docs/work_units/parity-deviations-register.md)

## 2. `NoteForYourself` Cross-Run Card Persistence

Status: intentional runtime-supported future-run effect

Policy:

- `NoteForYourself` is the event matching the “crack in the wall / leave a card for future runs” description.
- The canonical runtime behavior is:
  - read the note
  - claim the stored card
  - choose one current deck card to save for a future run

State-boundary decision:

- each simulation root receives the stored card through `ProfileSnapshot`
- the event emits `ProfileUpdate::StoreNoteForYourselfCard`; the core never mutates process-global profile state
- the default profile card is `IronWave`
- it is surfaced through the canonical event reward + deck-selection flow rather than a side channel

Why this shape:

- it keeps the mechanic inside the same typed event/reward runtime as the rest of the engine
- callers can persist or isolate profile updates explicitly without cross-root contamination

Canonical surfaces:

- [`src/events/shrines.rs`](src/events/shrines.rs)
- [`src/run.rs`](src/run.rs)
- [`src/tests/test_event_runtime_wave21.rs`](src/tests/test_event_runtime_wave21.rs)

## 3. `Match and Keep!`

Status: canonical runtime implementation landed

Policy:

- `Match and Keep!` runs as a real indexed reveal / match minigame inside the canonical event runtime.
- The runtime exposes one decision per visible slot and returns the updated event state after each reveal or pair resolution.
- The board is always `12` cards / `6` pairs with `5` attempts, and ascension `15+` replaces the colorless uncommon pair with a second curse pair.

Why this choice:

- it keeps the minigame on the same event / decision / reward surfaces as the rest of the run engine
- it gives the agent a clean observable loop: pick an index, observe the revealed state, keep matching if memory is good
- it removes the old fake fixed-reward stopgap so parity claims can rely on actual gameplay state

Canonical surfaces:

- [`src/events/shrines.rs`](src/events/shrines.rs)
- [`src/run.rs`](src/run.rs)
- [`src/tests/test_event_runtime_wave19.rs`](src/tests/test_event_runtime_wave19.rs)
- Java oracle: `decompiled/java-src/com/megacrit/cardcrawl/events/shrines/GremlinMatchGame.java`

## 4. Core Legality And Smoke Bomb

Status: Java-faithful core policy

Policy:

- the Rust core enumerates only actions that the game permits
- `SmokeBomb` is usable in ordinary combat, but is not a legal use action in boss combat or while an enemy has Java's `BackAttack` flag
- a potion can still be discarded when its use is illegal
- curriculum restrictions such as "take no card rewards" belong in a future consumer-side action mask, never in core legality

Canonical surfaces:

- [`src/potions/mod.rs`](src/potions/mod.rs)
- [`src/tests/test_potion_runtime_wave7.rs`](src/tests/test_potion_runtime_wave7.rs)
- [`src/tests/test_potion_runtime_action_path.rs`](src/tests/test_potion_runtime_action_path.rs)
- Java oracle: `decompiled/java-src/com/megacrit/cardcrawl/potions/SmokeBomb.java`

## 5. Process-Global RNG Inputs

Status: explicit oracle boundary

Policy:

- dungeon-owned RNG streams are derived and advanced exactly like Java
- libGDX `MathUtils.random` and the default `Collections.shuffle` LCG are separate process-global streams
- deterministic simulation constructors use documented defaults; oracle replay must inject captured raw global states
- a missing desktop ambient-state witness is reported as absent evidence, never treated as a Java match

Canonical surfaces:

- [`src/seed.rs`](src/seed.rs)
- [`src/trace/oracle_v2.rs`](src/trace/oracle_v2.rs)
- [`../docs/work_units/oracle-state-v2.md`](../../docs/work_units/oracle-state-v2.md)
