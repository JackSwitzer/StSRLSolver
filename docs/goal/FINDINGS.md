# FINDINGS — Oracle-Surfaced Parity Gaps

Running list of concrete divergences found by the trace oracle / decompile audit, most-actionable first. Each is a target for the `/goal` grind with the exact source citation. Clear an item by fixing + adding a smart test (or trace-matching it) and moving it to "Resolved".

## F1 — RESOLVED: Jaw Worm logic and secondary RNG draws

`roll_jaw_worm` now follows the three Java branch windows and consumes the
conditional `randomBoolean` draw from the shared AI stream. The source-derived
boundary and stream-order proofs live in `test_ai_rng_parity`.

## F2 — RESOLVED: all 13 run RNG counters are observable

`RunEngine::rng_counters()` now emits every canonical persistent, floor, map,
and Neow stream outside combat and overlays all seven combat-owned streams while
a fight is active. Counters are signed so Java `int` overflow remains traceable.

## F3 — RECONCILED: enemy AI RNG threading is already implemented

The 2026-04-17 audit §1.1 "enemies use no RNG, all deterministic" is **stale**. Commit `f0a0bfd4` added `enemies::roll_next_move(enemy, ai_rng)` (consumes `ai_rng.random(99)`), dispatched per-enemy across all 4 acts (`enemies/act1..4`), and it's wired in `combat_hooks.rs:516,568`. So U08 is *verify per-enemy roll parity against decompiled getMove* (F1-style), not *add RNG threading*. Update the audit reference when closing U08.

## F4 — INTENTIONAL DEVIATION; oracle remint required: Neow always exposes four choices

The simulator intentionally exposes all four seeded Neow choices on every run,
while Java progression can expose only two. The old smoke golden therefore
cannot establish an index mapping by position alone. New traces must carry the
selected option payload and the deviation must remain explicit; the underlying
four category constructors and RNG consumption are source-tested.

## F5 — RESOLVED: potion empty-slot representation matches Java

Rust trace emission now serializes empty slots as Java's stable `"Potion Slot"`
ID both outside and during combat, with source-derived regression coverage.

## F6 — RESOLVED: every serialized v1 record field participates in the differ

`trace.rs::record_field_diffs` compares record identity and action, every RNG
counter, player scalars/powers/orbs, enemy identity/scalars/intents/powers/move
history, all piles, relic IDs/counters, and potion slots. One-field corruption
fixtures cover each nested family while preserving RNG-first diagnosis.

## F7 — RESOLVED ON RUST; superseded by the v2 Java adapter handoff

- Resolved: `PostState.rng` and the differ use `BTreeMap<String, i64>`, so Java's
  `-1` null-stream marker and signed counter overflow round-trip correctly.
- V2 serializes the canonical `GameAction` directly, covers all 27 current run
  variants, and has no stop-condition mismatch. The exhaustive schema test pins
  every serialized variant to `docs/work_units/script-schema-v2.md`.
- V1 remains read-only for the existing smoke golden. Its CAMPFIRE and
  `max_actions` nits must not be carried into the Java v2 adapter.

## F9 — OPEN ON RECORDER: semantic v2 actions and causal checkpoints

Rust now owns the language-neutral `sts.oracle_state` v2 projection, strict
schema validation, canonical actions, deterministic checkpoints, and offline
multi-sitting bundle intake. The current Java recorder still emits UI commits,
omits nested selections and leave/skip actions, and sometimes captures before
the action queue settles. Those omissions are reported as action-mapping or
coupled-checkpoint gaps, never matches. The current operator handoff is
`data/traces/requests/wave3-recorder-needs.md`; no agent may manufacture or
write the protected Java corpus.

## F10 — OPEN ORACLE INPUT: process-global ambient RNG state and draw witness

Java does not derive `MathUtils.random` or the default
`Collections.shuffle` RNG from the dungeon seed, and it does not reset either
stream between runs in one process. Rust implements both generators exactly
and exposes a constructor accepting their captured states; the deterministic
seed-zero defaults are simulation policy, not a Java oracle witness.

An initial state alone is not a sufficient desktop oracle. `MathUtils.random`
appears in 327 Java source files and is shared by presentation-only animation,
audio, and dialogue calls as well as gameplay-facing selectors such as shop
card identity. Render cadence can therefore perturb the next semantic ambient
draw even when the dungeon RNG streams remain identical. Full-run desktop
certification requires settled checkpoints to include both ambient states (or
an equivalent ordered ambient-draw witness) so presentation entropy is not
mistaken for a simulator defect. Headless training remains deterministic under
the documented simulator initialization policy.

## F11 — RESOLVED: process-global RNG lifecycle across shops and run reset

Successful card purchases now consume Java's speech-timer, voice, buy-message,
side, and position `MathUtils.random` draws; relic and potion purchases consume
the four shared speech draws. `RunEngine::reset` preserves both process-global
RNG states while rebuilding every dungeon-owned stream from the new seed.
Masked Bandits payment also consumes one ambient bandit-selection draw per
current gold before removing it, which is proven to alter a later Courier
refill if omitted. Consecutive Courier purchases and reset ownership have
source-derived action-level tests.

## F12 — RESOLVED: targetable enemy catalog and canonical identity

The engine now derives a 66-entry targetable monster catalog from the verified
68-row ledger, excluding only the two non-`AbstractMonster` Hexaghost visual
helpers. Compatibility aliases normalize before construction, runtime state and
exports use each Java `ID` constant, every catalog member constructs and rolls,
and unknown IDs fail closed instead of becoming a fabricated generic attack.

## F13 — RESOLVED: trace/checkpoint projection honesty

Intent damage retains Java float ordering through the final floor, repeated
Searing Blow upgrades validate against `misc`, unknown `?` rooms use settled
recorder evidence instead of being forced to EVENT, Pandora keeps paired deck
identity aligned, and checkpoint semantics revision v2 rejects older snapshots.

## F14 — RESOLVED: mystery-combat room identity survives to rewards

`EventRoom` monster rolls now store the concrete `MonsterRoom` identity with
the active combat instead of re-reading the map's still-visible `?` symbol at
victory. The typed identity replaces the duplicate boss boolean, serializes in
causal checkpoints, and proves that Prayer Wheel creates two card rewards after
a mystery-room hallway fight.

## F15 — OPEN COVERAGE: uninterrupted canonical Neow-to-Heart replay

The canonical action audit found no missing Watcher A0 player-decision family:
`GameAction`, legal-action enumeration, and `step_game` cover Neow, pathing,
combat and choices, rewards, potions, shops, events, campfires, chests,
transitions, Act 4, and terminal state. However, no test or script currently
drives that surface uninterrupted from Neow to the Heart. The core action test
stops at the first combat, the v2 smoke script ends incomplete on floor 1, and
the Act 4/Heart proof starts from test-injected Act 3 state and forces combat
outcomes.

Queued proof: `watcher_a0_v2_neow_to_heart_replays_deterministically`, using a
complete semantic recorder script after F9 is fulfilled. It must replay twice
to byte-identical transition envelopes and may not use debug state injection or
forced combat outcomes.

## F8 — April 2026 parity stack: unmerged fix quarry (tag `april-2026-parity-stack`)

Eleven stacked April PRs (#138-148, now closed) contained ~4,600 lines of unmerged engine parity fixes — enemy AI across acts 1-4 (`enemies/act1..4.rs`, `enemies/mod.rs`), powers dispatch wiring, damage-pipeline routing, CorruptHeart A0, snapshot determinism — plus ~3,700 lines of source-cited tests. They pre-date the verification-sweep contract, so they were **never merged**: they are another unverified draft, and main's engine moved since April (conflicts guaranteed).

**How to use when verifying a related ledger row** (enemies, powers, damage): consult the quarry as a *hint*, then verify from decompiled source as always:
```bash
git diff main...april-2026-parity-stack -- packages/engine-rs/src/enemies/act2.rs   # prior fix attempt
git show april-2026-parity-stack:docs/work_units/parity-deviations-register.md      # April D-register (main's has since diverged)
```
The per-area April reports are preserved on the tag rather than in the live documentation tree. Read one with, for example, `git show april-2026-parity-stack:docs/work_units/audit-reports/enemies-act1.md` (other reports include `powers-buffs-debuffs.md`, `damage-engine-flow.md`, and `watcher-cards.md`). A hint that disagrees with the decompiled source loses; a hint that agrees saves you the derivation.
