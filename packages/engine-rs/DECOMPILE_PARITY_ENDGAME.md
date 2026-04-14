# Decompile-Backed Parity Endgame

Last updated: 2026-04-14  
Branch: `codex/universal-gameplay-runtime`

## Goal

Use the local Java oracle at:

- `/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src`

as the semantic source of truth for the remaining parity tail, while keeping the Rust runtime owner-aware, typed, and RL-friendly.

We copy Java semantics and timing, not Java architecture.

## What Counts As Done

A migration slice only counts as done when all of the following are true:

- production behavior runs through the canonical runtime/decision surface
- the slice has focused engine-path test coverage
- tests cite the exact Java source file(s) used as the parity oracle
- no helper-path-only assertion is carrying that behavior
- hidden counters/flags live in runtime state, not mirrored through accidental status hacks

If a missing primitive blocks a slice, the next-best acceptable state is:

- the primitive gap is named explicitly
- a focused engine-path test lands as `#[ignore]` or a queued blocked case
- the test cites the exact Java file proving expected behavior

## Verification Substrate

Use the repo-standard wrapper for all focused verification:

- `/Users/jackswitzer/Desktop/SlayTheSpireRL/scripts/test_engine_rs.sh`

Minimum worker acceptance:

- `./scripts/test_engine_rs.sh check --lib`
- `./scripts/test_engine_rs.sh test --lib --no-run`
- one or more focused engine-path suites for the owned slice

## Translation Rules

Every migrated legacy registrar element should translate to one canonical runtime-backed definition with:

- domain
- schema
- handlers
- state fields
- canonical program/effect ops
- optional typed complex hook only if a missing primitive still blocks full declarative expression

Mapping rules:

- Java hook methods map to typed event kinds, not string-tag side channels
- Java action queue behavior maps to `EffectOp` or `DecisionFrame`, not inline engine match logic
- Java private counters/booleans map to runtime `EffectState`
- Java timing/order semantics should be preserved exactly where it affects parity or RL branching
- cards, relics, powers, potions, events, and rewards should all produce engine-path tests through the same runtime surfaces

## Remaining Primitive Families

The remaining hook-heavy tail mostly compresses into these primitive families:

1. Card-play phase fidelity
2. Zone selection and batch pile movement
3. Generated-choice and discover-style random generation
4. Orb lifecycle and top-of-pile free play
5. Damage / debuff / HP-loss follow-up payloads
6. Reward / event / map transition edge branches

## Phase Order

### Phase 0. Verification substrate

Already landed:

- wrapper-based local Rust verification
- Java decompile cache refresh path

### Phase 1. Card-play phase fidelity

Target semantics:

- `onPlayCard`
- `onUseCard`
- `onAfterUseCard`
- `onAfterCardPlayed`
- replay-window semantics

Priority entities:

- `Time Warp`
- `Echo Form`
- `Double Tap`
- `Burst`
- `Panache`
- `Thousand Cuts`
- `Orange Pellets`
- `Pocketwatch`

### Phase 2. Zone selection and batch pile movement

Target primitives:

- select from zone
- move selected card
- move card batch
- after-batch resolution

Priority entities:

- `Headbutt`
- `True Grit`
- `Burning Pact`
- `Second Wind`
- `Fiend Fire`
- `Storm of Steel`
- `Purity`
- `Secret Technique`
- `Violence`

### Phase 3. Generated-choice and discovery

Target primitives:

- random generation to hand/discard/draw
- discover-style choose-one generation
- deterministic RNG labels by pool and choice index

Priority entities:

- `Chrysalis`
- `Metamorphosis`
- `Transmutation`
- `Attack Potion`
- `Skill Potion`
- `Power Potion`
- `Colorless Potion`

### Phase 4. Orb lifecycle and pile-play

Target primitives:

- orb channel/evoke/passive/slot-change events
- top-of-pile free play semantics
- instance-scoped cost mutation

Priority entities:

- `Streamline`
- `Chaos`
- `Fission`
- `Reboot`
- `Barrage`
- `Liquid Memories`
- `Distilled Chaos`
- `Cracked Core`
- `Frozen Core`
- `Emotion Chip`

### Phase 5. Damage / debuff / HP-loss follow-up

Target payloads:

- outgoing damage adjusted
- incoming damage after block
- HP lost
- debuff applied
- enemy death
- victory

Priority entities:

- `Envenom`
- `Sadistic Nature`
- `The Specimen`
- `Red Skull`
- `Centennial Puzzle`
- `Preserved Insect`
- `Du-Vu Doll`
- `Girya`
- `Slaver's Collar`

### Phase 6. Event / reward / map transition edges

Target semantics:

- scripted combat continuation
- map-jump / boss-room transitions
- minigame/random-table branches
- reward branching that stays on the canonical reward runtime

Priority entities:

- `Colosseum`
- `Cursed Tome`
- `Secret Portal`
- `Bonfire Elementals`
- `Wheel of Change`

## Current Active Worker Wave

These bundles are intentionally disjoint by write scope:

- dead-export cleanup follow-up centered on `src/powers/registry.rs`, `src/powers/mod.rs`, and `src/effects/dispatch.rs`
- read-only remaining-tail regroup and broad-audit preparation

Each worker must return:

1. changed files
2. commands run plus results
3. any remaining blockers mapped to ignored or queued tests

## Immediate Blocker Map

These are the currently verified card-tail clusters that should drive the next primitive waves once the active bundles land:

- Silent discard/post-choice cluster:
  - `Alchemize`
  - `Nightmare`
  - `Reflex`
  - `Tactician`
- Ironclad exhaust/top-play cluster:
  - `Burning Pact`
  - `Dual Wield`
  - `Feed`
  - `Fiend Fire`
  - `Reaper`
  - `Second Wind`
- Defect frost/order cluster:
  - `Blizzard`
  - `Fission`
  - `Reboot`
  - `Scrape`
  - `FTL`
- Watcher decision/payload cluster:
  - `Deus Ex Machina`
  - `Lesson Learned`
  - `Omniscience`
- Colorless utility/scaling cluster:
  - `Enlightenment`
  - `Ritual Dagger`
  - `Violence`

The shared primitive themes behind those clusters are now clear:

- discard-then-resolve post-choice sequencing
- filtered random attack fetch from draw
- typed exhaust-all / exhaust-per-hit loops
- draw-to-N and no-attacks-in-hand checks
- enemy-HP threshold kill
- turn-only hand cost reduction
- post-damage amount resolution from unblocked damage
- draw-pile-size and card-owned misc scaling
- event-local persistent search-state ramp and elite-combat continuation for `Dead Adventurer` remains fully modeled on the typed path

## Test Policy

Preferred assertions:

- exact timing/order checks
- exact legality checks
- exact reward/choice sequencing
- deterministic replay with the same seed and the same decision sequence
- exact per-instance mutation behavior where Java tracks the played/generated card instance

Avoid counting these as parity evidence:

- helper-only oracle tests
- registry-presence tests without behavior
- tests that assert a placeholder reason but never exercise production behavior

## Final Merge Bar

Before opening the final merge-quality PR:

- no live production callsites should depend on legacy dispatch/oracle paths
- the remaining blocked-event count should be zero for supported content
- the live potion fallback in `CombatEngine::use_potion` should stay gone
- the hook tail should be reduced to cases with a documented primitive gap or a truly irreducible hook
- scorecard and focused suites should reflect the production path, not test-only shims
