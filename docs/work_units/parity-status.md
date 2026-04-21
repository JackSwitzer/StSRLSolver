# Parity Status

Last updated: 2026-04-17
Branch: `claude/sharp-solomon-a1c9ec`

This work unit is the standing parity readout for the Watcher A0 combat-first training rebuild. It is paired with the recorded-run replay launched 2026-04-17 against `runs/WATCHER/1776347657.run`.

## 0. Critical open bug: enemy AI uses no RNG (Rust diverges from Java)

**Status: confirmed by independent investigation 2026-04-17. Not in audited matrix.**

In Java, every `AbstractMonster.rollMove()` consumes one value from `AbstractDungeon.aiRng` (`decompiled/java-src/com/megacrit/cardcrawl/monsters/AbstractMonster.java:466`). That value is passed as `num` to `getMove(int num)`, where each monster uses `randomBoolean(p)` calls to branch its intent probabilistically (e.g., `JawWorm.java:146-181`, plus ~20+ enemies including the Chosen, Sentries pair-decisions, multi-enemy intent ordering).

In Rust:
- `EnemyCombatState` has **no rng field** (`packages/engine-rs/src/state.rs:142-156`).
- `roll_next_move()` takes **no RNG parameter** (`packages/engine-rs/src/enemies/mod.rs:815`).
- `roll_jaw_worm()` is **100% deterministic** with zero RNG calls (`packages/engine-rs/src/enemies/act1.rs:11-21`). Same pattern across all enemy roll functions.
- `CombatEngine.rng` exists (`packages/engine-rs/src/combat_engine.rs:117`) but is not threaded into enemy AI.

Impact: any combat with a probabilistic enemy decision diverges from Java intent sequences. Multi-enemy combats (3 Sentries, Centurion+Healer, Lots of Slimes) will additionally diverge because Java's shared-stream order coupling is absent. Single-enemy hardcoded patterns happen to produce the most-common branch and so the existing `test_*_parity.rs` suites pass — but they were not asserting probabilistic intent variance. The "2189/2189" freeze does not cover this.

Why this matters tonight: in our recorded-run replay (Act 1 combats only since the encounter catalog stops there), the solver "outperformed" the human on Jaw Worm (0.3 vs 6 HP loss) and Guardian (0 vs 9). Some of that delta is genuine PUCT skill; some is fighting deterministic dummies that always pick the same intent. Cannot disentangle without the fix.

Fix scope (estimate): structural, ~5–10 files, ~200+ lines.
1. Thread `&mut StsRandom` from `CombatEngine` into `roll_next_move` and all enemy roll functions (`enemies/mod.rs:815` + `act1.rs`/`act2.rs`/`act3.rs`/`act4.rs`).
2. Restore `num = rng.random(99)` seeding at each rollMove call (`combat_hooks.rs:511`).
3. Re-implement probabilistic branching for JawWorm, Chosen, and the ~20+ affected enemies using the passed `num`.

This belongs at the front of the parity work queue. Until it lands, treat any solver-vs-recorded delta as a lower bound on parity drift.

## 1. Engine vs Java decomp parity

Headline: **Functionally complete for Watcher A0 Act 1 combat. Not 100% across all surfaces.**

What is closed on the audited matrix (see `docs/research/engine-rs-audits/AUDIT_PARITY_STATUS.md:16-29`, `INCONSISTENCY_REPORT.md:12-30`):

- Watcher cards: 100% of the registered Watcher catalog runs through the typed runtime path; broad class freeze `test_cards_watcher` green (`AUDIT_PARITY_STATUS.md:129`).
- Colorless, Curses, Status: 100% on the audited matrix.
- Act 1 enemies: 22 of 28 registered with 100% critical-path coverage for the Watcher A0 hallway/elite/boss encounter set.
- Act 1 events: 100% (including `NoteForYourself`, `Match and Keep!`, `Scrap Ooze`) on the canonical event runtime (`INCONSISTENCY_REPORT.md:27`).
- Pure Water and the common Act 1 relic surface: closed.
- Core combat powers: Strength, Dexterity, Vulnerable, Weak, Frail, plus enemy Anger, Curl Up, Sharp Hide, Mode Shift.
- Watcher-specific powers: Rushdown, Nirvana, Equilibrium, Mark, Deva Form, Devotion.
- Final broad freeze: `2189 / 2189` green (`AUDIT_PARITY_STATUS.md:47`).

What is in the tail (cite `docs/research/engine-rs-audits/COMPLEX_HOOK_AUDIT.md:25-30`):

- Card-hook surface: 71 of 352 card files still execute through the legacy `complex_hook` fallback. The `complex_hook` count for raw *public* gameplay-gap files is `0` (`INCONSISTENCY_REPORT.md:55-56`); the 71 number is the wider authoring-side migration backlog.
- The "5 truly complex" cards still pending dedicated runtime primitives: Wish, Nightmare, Omniscience, Lesson Learned, Time Warp. Lesson Learned is the only one that materially hits Watcher decks and should be considered a known parity risk for any seed that draws it from a card reward (`COMPLEX_HOOK_AUDIT.md:292`).
- Acts 2-3: relic coverage ~54% overall, power coverage ~50% overall. Act 1 is the high-confidence zone; Acts 2-3 surface is not yet inside the audited matrix.

Intentional deviations (cite `packages/engine-rs/DESIGN_DECISIONS.md:8-28`, `:30-57`, `:59-79`):

- Neow always exposes 4 choices regardless of vanilla Java progression gating (RL-facing deviation, `DESIGN_DECISIONS.md:8-28`).
- `NoteForYourself` future-run storage is canonical inside the runtime process rather than external profile-save persistence (`DESIGN_DECISIONS.md:30-57`).
- `Match and Keep!` runs as an indexed reveal/match minigame on the typed event runtime (`DESIGN_DECISIONS.md:59-79`).

Likely engine-side issues to surface during the recorded-run replay:

- Time Eater turn-counter mechanic if a recorded boss replay reaches it.
- Heart multi-phase sequencing (out-of-scope for Act 1 but the contract may still trip on shared power lookups).
- Act 2 elites that draw from the `complex_hook` tail.

## 2. Training-side alignment

Headline: **Training observation contract alignment: ~0% gap items still open.**

Open items (cite `docs/research/engine-rs-audits/INCONSISTENCY_REPORT.md:254-261`):

- Align training with the live Neow surface; current default is `skip_neow=True`.
- Document and version the Rust ↔ Python observation contract.
- Add a training-side restriction overlay for curriculum rules (e.g. "no card rewards") so they live above the engine instead of inside it.

Currently working:

- `CombatPuctConfigV1` schema v1 is live with hallway/elite/boss presets at `packages/engine-rs/src/training_contract.rs:352-395`.
- Training contract test passes: `./scripts/test_engine_rs.sh test --lib training_contract`.
- `RL combat surface | 98%` per audit scorecard (`AUDIT_PARITY_STATUS.md:24-25`); the remaining 2% is the contract-versioning and restriction-overlay work above.

## 3. Live combat-by-combat dashboard

The recorded-run replay built tonight in `packages/training/run_replay.py` walks the WATCHER A0 winning run at `runs/WATCHER/1776347657.run` (Apr 16, 23 combats, victory). It emits per-floor `combat_solved`, `combat_failed`, and `combat_unsupported` events that SpireMonitor's `RecordedRunReplayView` consumes.

Per-combat columns to populate as the replay runs:

| floor | encounter | recorded_hp_loss | solver_hp_loss | status | search_visits | convergence_flag |
| ---: | --- | ---: | ---: | --- | ---: | --- |
| ... | ... | ... | ... | solved / failed / unsupported | ... | ... |

Pass criterion (per combat):

- `solver_hp_loss <= recorded_hp_loss + max(5, 0.1 * max_hp)`

Failures reported by this dashboard are the next concrete parity gaps to scope. This section is living and updates with each replay pass; treat the table as the working surface, not the conclusion.
