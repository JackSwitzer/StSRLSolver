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

## F4 — Neow option index maps to a different blessing than Java

smoke-neow-floor1 (seed 57554006466, choice 1): Java → maxHP 79, gold 99; Rust → maxHP 72, gold 199. Rust's `NEOW` action index does not select the same blessing Java's option 1 does. Either the option ordering differs or the Rust Neow model doesn't apply the maxHP-boost blessing. Cross-ref the existing intentional-deviation register (Neow exposes 4 options) before deciding fix vs mask. This is U09.

## F5 — Potion empty-slot representation differs (cosmetic)

Java emits `"Potion Slot"` for an empty potion slot; Rust emits `""`. Normalize one side (recommend Rust emit the same placeholder, or the differ treat both as empty) so it stops appearing as a divergence. Trivial; do with F2.

## F6 — Differ blind spots: powers, move_history, relic counters not compared

`trace.rs` `record_field_diffs` compares rng/player scalars/enemy scalars+intent/piles/relic *ids*/potions — but **not** `player.powers`, `enemies[].powers`, `enemies[].move_history`, `player.orbs`, or `relics[].counter` (GOAL DoD 1 explicitly names relic counters). A divergence confined to those fields reports `match`. Both sides already emit the data (TraceWriter.java + `build_post_state`), so this is compare-side only. Two pre-requisites before enabling: (a) Rust `build_post_state` hardcodes `counter: -1` for every relic — engine relic-counter tracking must land first or every counting relic diverges on noise; (b) power id vocabularies must be reconciled (Java `AbstractPower.ID` e.g. `"Vigor"` vs Rust `status_name`). Enable field-by-field as each becomes clean, rng-first order preserved.

## F7 — Java/Rust harness contract nits (flagged, not yet biting)

- Resolved: `PostState.rng` and the differ use `BTreeMap<String, i64>`, so Java's
  `-1` null-stream marker and signed counter overflow round-trip correctly.
- `Script.Action.choice` is `Integer` in the Java harness, while the Rust `TraceAction::Campfire` and the TOOLING T2 example use a string (`"REST"`). CAMPFIRE isn't implemented in the harness yet (`execute()` rejects it); whoever adds it must reconcile the type first.
- Rust `ScriptStopCondition.max_actions` has no Java-side counterpart; scripts relying on it will produce longer Java goldens than Rust replays. Use `max_floor` (semantics: stop once floor *exceeds* max_floor — actions on the max floor still run) or trim the action list.

---

## Resolved

_(none yet)_

## F8 — April 2026 parity stack: unmerged fix quarry (tag `april-2026-parity-stack`)

Eleven stacked April PRs (#138-148, now closed) contained ~4,600 lines of unmerged engine parity fixes — enemy AI across acts 1-4 (`enemies/act1..4.rs`, `enemies/mod.rs`), powers dispatch wiring, damage-pipeline routing, CorruptHeart A0, snapshot determinism — plus ~3,700 lines of source-cited tests. They pre-date the verification-sweep contract, so they were **never merged**: they are another unverified draft, and main's engine moved since April (conflicts guaranteed).

**How to use when verifying a related ledger row** (enemies, powers, damage): consult the quarry as a *hint*, then verify from decompiled source as always:
```bash
git diff main...april-2026-parity-stack -- packages/engine-rs/src/enemies/act2.rs   # prior fix attempt
git show april-2026-parity-stack:docs/work_units/parity-deviations-register.md      # April D-register (main's has since diverged)
```
The per-area April reports are preserved on the tag rather than in the live documentation tree. Read one with, for example, `git show april-2026-parity-stack:docs/work_units/audit-reports/enemies-act1.md` (other reports include `powers-buffs-debuffs.md`, `damage-engine-flow.md`, and `watcher-cards.md`). A hint that disagrees with the decompiled source loses; a hint that agrees saves you the derivation.
