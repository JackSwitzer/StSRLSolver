# Decompile-Backed Parity Endgame

Last updated: 2026-04-14  
Branch: `codex/universal-gameplay-runtime`

## Goal

Use the local Java oracle at:

- `/Users/jackswitzer/Desktop/SlayTheSpireRL/decompiled/java-src`

as the semantic source of truth for the remaining parity tail, while keeping the Rust runtime owner-aware, typed, and RL-friendly.

We copy Java semantics and timing, not Java architecture.

Canonical audit output for the current tail:

- [`INCONSISTENCY_REPORT.md`](./INCONSISTENCY_REPORT.md)

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

## Current Checkpoint

Recent accepted card/runtime slices moved the following cards onto the typed primary surface:

- `FTL`
- `Bane`
- `Feed`
- `All-Out Attack`
- `Alchemize`
- `Reaper`
- `Violence`
- `Ritual Dagger` damage body
- `Escape Plan`
- `Malaise`
- `Lesson Learned`
- `Reboot`
- `Fission`
- `Blizzard`
- `True Grit`
- `Second Wind`
- `Burning Pact`
- `Dual Wield`
- `Fiend Fire`
- `Nightmare`
- `Scrape`

The raw public-card tail is `3` files, and the honest unresolved gameplay-gap tail is `0` after excluding the runtime-backed non-play cleanup shells `Reflex`, `Tactician`, and `Deus Ex Machina`. The separate shared card modules (`cards/mod.rs`, `cards/curses.rs`, `cards/status.rs`) are tracked as registry/support surfaces, not as unresolved public-card files.

Fresh audit framing:

- supported-scope gameplay blockers: `0`
- unsupported blocked event branches still present in source: `1` (`Scrap Ooze`)
- total ignored tests still present in `src/tests`: `92`
- ignored-test classified buckets: `26` active parity blockers, `50` stale solved/noisy, `11` post-merge enhancements, `4` cleanup-only/accounting, `1` unsupported
- `Match and Keep!` is now temporarily routed through the canonical event reward runtime as a fixed `Rushdown+` / `Adaptation+` reward; it is no longer unsupported-source debt, but it remains approximation debt until the Java minigame lands
- the remaining live semantic debt is now concentrated in generated-choice payload fidelity, potion legality/choose-one edges, `Emotion Chip` timing, `Neow's Lament`, `Scrap Ooze`, and a post-merge enhancement tail documented in [`INCONSISTENCY_REPORT.md`](./INCONSISTENCY_REPORT.md)

The remaining explicit blockers from those recent tiny-primitive waves are still Java-cited and intentional:

- none for the current tiny-primitive wave; `Enlightenment` base is now on the typed turn-only cost path

The recent non-play cleanup also retired the stale blocker sentinels for:

- `Reflex`
- `Tactician`
- `Deus Ex Machina`

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

## Current Audit Wave

The active endgame work is now audit-first rather than primitive-first:

- broad matrix verification over the stable green-core suites
- ignored-test classification into active blocker vs stale solved vs cleanup-only vs post-merge enhancement
- Java semantic review of the remaining real mismatch families
- scope-honesty reconciliation for unsupported branches and cleanup shells
- training appendix preparation for the post-merge training-system rewrite

## Immediate Blocker Map

These are the currently verified remaining behavior clusters:

- Watcher shared play-tail cluster:
  - `Third Eye`
  - `Foreign Influence`

The shared primitive themes behind those clusters are now clear:

- runtime-backed non-play trigger shells are no longer treated as unresolved gameplay gaps once engine-path proof exists

- shared play-card continuation after a choice opens
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
