# Audit 6 — Stage B tail bitrot cross-ref

**Status:** CLEAN (no bitrot requiring REOPEN; register tracks all material gaps; ~230 Stage B tail still parked)
**Currently-open rows (register):** 137 open + 4 deferred + 2 partial-fix (of 181 total D1–D181; 38 closed)
**Stage B candidates for Cycle 8 promotion:** 8 (below)

## Promotion candidates (top 8, Watcher A0 impact)

1. **Watcher stance — WaveOfTheHand early-clear** — Java `WaveOfTheHandPower.atEndOfRound` removes at end-of-round; Rust clears at next player-turn start (`packages/engine-rs/src/engine.rs:1114`), so enemy-turn block gains still apply Weak. Java: `decompiled/java-src/com/megacrit/cardcrawl/powers/watcher/WaveOfTheHandPower.java:atEndOfRound`. P1. WS12.
2. **Watcher latent dead-dispatch cluster — WS7/WS8/WS9/WS10 umbrella** — `process_start_of_turn` / `process_end_of_turn` at `packages/engine-rs/src/powers/buffs.rs:620,704` never called from `engine.rs`; chains `WrathNextTurn`, `EndTurnDeath`, `LiveForever` (AngelForm), `NoSkills`. All Watcher-flavored; all latent but will silently no-op when any future effect sets the status. Recommend a single umbrella D-row (mirrors D70-Equilibrium handling). P1 structural.
3. **Act 1 — Cultist Strength ascension scaling** — `packages/engine-rs/src/enemies/act1.rs::roll_cultist` hardcodes `strAmt=3`; Java `Cultist.java:strAmt=3 (base), 4 (A2+), 5 (A17+)`. E3A1 Stage B. P1 (Cultist is opener for ~25% of A1 first-combat fights, Watcher A0 opener damage curve off).
4. **Act 1 — GremlinNob Bellow ascension cap** — E15A1; Rust `enemies/act1.rs:140` gives `FRAIL 1` regardless; Java `GremlinNob.java` A17+ `FRAIL 2` on Bellow. Strip cards + Defensive Stance scaling fights. P1.
5. **Act 1 — Hexaghost Inferno A19+ scaling** — E6A1; Rust hardcodes 3-damage hits in `roll_hexaghost`; Java Inferno scales to 4-damage hits at A19+ plus Burn count. Common Act 1 boss. P1 (Watcher A0 always encounters on training-path).
6. **Potions — Distilled Chaos was promoted as D178 but Distilled Chaos + SwiftPotion return-to-hand (PT5)** — SwiftPotion `Duplication` status never consumed. `packages/engine-rs/src/potions/defs/duplication_potion.rs:4` sets `sid::DUPLICATION` but no `OnCardPlayed` hook decrements. Balance impact: Silent-build damage doubling lost. P1 (PT5).
7. **Potions — Cunning Potion summons plain Shiv instead of Shiv+** — PT4; `potions/defs/cunning_potion.rs` adds `AddCard("Shiv", ...)` instead of `Shiv+`. Java calls `.upgrade()` after construction. Watcher rarely uses but Silent-Watcher dual-class borderline; P2.
8. **DamagePipeline — Thorns retaliation in attack pipeline (DM2)** — Java `ThornsPower.onAttacked` routes through `DamageInfo` calculator (passes attacker powers); Rust `combat_hooks.rs:240-251` applies raw thorns. Distinct from D170 (attacker-Intangible cap) — DM2 covers Vulnerable/Strength layering on thorns. P1 (Apparition+Byrds, Spiker, Reptomancer snakes for Watcher).

**Honourable mentions (11–15):** PT3 Cultist Potion Ritual dead; P17 EnvenomPower dual-route (absent); P40 Barricade block-retention; R17–R21 missing relics tier (Dolly's Mirror transform, Necronomicon first-Attack, Peace Pipe, Inserter energy-every-2); OC4 Searing Blow HP-scaling damage formula.

## Deferred D155 sanity

Register row D155 verbatim (`docs/work_units/parity-deviations-register.md:229`):

> **closed (alternation half) 2026-04-21** — Cycle 4b-rest fixed the turn-by-turn alternation by wiring Java's `lastIndexOf(this) % 2` positional opener (see D156)… **Outstanding substructure:** the Rust `move_ids::SENTRY_BOLT` / `SENTRY_BEAM` constants remain semantically inverted from Java (Rust `SENTRY_BEAM` still carries DAZE + 9 damage; Java BEAM is damage-only, Java BOLT is Dazed-only with no damage). Renaming the constants + splitting the 9-damage-plus-DAZE combo is deferred to a dedicated follow-up (cascades through SpireMonitor labels, `test_enemies.rs`, `test_enemy_ai.rs`, and `run.rs:1249` staging). Commit `7a9045c4` (Cycle 4b-rest / PR #145).

TODO is present and clear: explicit scope (rename constants + split DAZE combo), explicit cascade list, closure gated on substructure split. **No REOPEN needed.**

## Currently-open rows enumerated (P0/P1 highlights)

**Open P0 (determinism + Act-boss critical):**
- D88 (P0) HolyWater gives 3 HolyWater cards instead of 3 free energy
- D90 (P0) MalleablePower no atEndOfTurn reset
- D91 (P0) `deal_damage_to_player` pipeline bypass *(triage says closed Cycle 5, register still open — STALE STATUS but not bitrot)*
- D92 (P0) 44 orphan test files (~230 tests)
- D93 (P0) `combat_engine_from_snapshot` drops orbs + combat_over
- D124 (P0) Pressure Points HP-loss bypass *(triage says closed Cycle 5, register still open — STALE STATUS)*
- D162 (P0) CombatSnapshotV1 drops ai_rng (F1)
- D163 (P0) EnemySnapshotV1 drops move_history (F2)
- D164 (P0) combat_state_hash ignores RNG (F3)
- D165 (P0) Snapshot roundtrip test coverage gap (F4)

**Open P1 (Watcher A0 path, Act enemies, combat-hooks):**
D1, D58, D66, D94, D95, D100, D102, D111 *(wait — D111 now closed per register check)*, D123, D125, D126, D127, D128, D132, D133, D134, D135, D136, D137, D138, D139, D141, D142, D145, D146, D147, D148, D149, D150, D151, D153, D157, D160, D161, D166–D181.

**Deferred (4):** D131 (JawWorm sub-roll), D159 (test consolidation), plus 2 partial-fix (D59 `9a337a35`, D87 `a2104416`).

Full open list: see tool output — 137 rows total.

## Bitrot risk

**Low.** Register actively tracks ~230 un-promoted tail via Stage B per-area reports as the disposition channel; the stage-b-crossref doc identified 2 stale-status rows (D91/D124) which are **docs drift not bitrot** — the fix landed, register tag didn't. Recommend a one-line register edit in the PR #138 merge commit to flip D91/D124 to closed with commit ref.

## Recommendation

**Ship PR #138** — register hygiene is adequate. Two nits worth folding in:
1. Flip D91/D124 status to `**closed** (Cycle 5)` with commit `10c34602` ref.
2. Add a single umbrella D-row for dead-dispatch cluster (WS7–WS10 + D70 Equilibrium) pointing at `powers/buffs.rs:620,704` so Cycle 8 has one promotion target instead of four.

No merge-blocking bitrot found.
