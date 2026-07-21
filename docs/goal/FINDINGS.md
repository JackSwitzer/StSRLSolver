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

## F9 — OPEN; human/Java boundary: v2 oracle projection and corpus mint

Rust v2 replay now accepts canonical action scripts and emits deterministic
causal `CoreCheckpoint` chains. A `CoreCheckpoint` contains the Rust engine's
private continuation representation, so it is not a language-neutral state
shape that Java can emit faithfully. The Java recorder still needs the v2
action adapter plus a frozen shared state/RNG projection before scarce human
sessions can mint reliable full-run goldens. The handoff is registered at
`data/traces/requests/watcher-a0-oracle-closure.json`; no agent may manufacture
or write the protected Java corpus.

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
