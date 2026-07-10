# Pre-Merge Triage — 2026-04-21 (v2, post-remediation)

**Audit fleet:** 7 parallel Opus subagents
**Branch audited:** `claude/cycle-7-final-audit` at `e38c3ec4`
**Scope:** verify closure of every Cycle 1-6 PR; hunt for regressions introduced during
remediation; cross-check the Stage B tail for bitrot.

## Top-line

**Ship PR #138.** No P0 or P1 blockers remaining. Two-point docs drift surfaced and fixed
in this commit (register rows D91/D124 still marked open despite Cycle 5 closure; D162-D165
likewise despite Cycle 2 closure). One silently-latent-correct row (D179) reclassified and
its test unignored.

## Remediation cycle ledger

| PR  | Cycle | Branch                                    | Closed rows                                    | Commit     | Tests |
|-----|-------|-------------------------------------------|------------------------------------------------|------------|-------|
| #139 | 1     | `claude/cycle-1-test-suite-refresh`       | 24 dup boss tests deleted; 9 weak asserts      | `ed0af936` | baseline → +5 red |
| #140 | 2     | `claude/cycle-2-determinism`              | F1/F2/F3/F4 (D162-D165)                        | `f30066fe` | +5 snapshot |
| #141 | 4c    | `claude/cycle-4c-writhingmass`            | D140                                           | `51e11483` | +1 (RED→GREEN) |
| #142 | 4b    | `claude/cycle-4b-ascension-scaling`       | D118 subset (Nob/Snecko/Spiker/SpireShield)    | `e5108bdf` | +5 ascension |
| #143 | 3     | `claude/cycle-3-powers-dispatch-wire`     | D70, D89, D111                                 | `f74dbaa1` | +6 dispatch + 4 defs |
| #144 | 4a    | `claude/cycle-4a-corruptheart`            | D143, D144                                     | `a5fc04b4` | +8 CorruptHeart |
| #145 | 4b-r  | `claude/cycle-4b-rest-enemy-ai`           | D152, D154, D155 (alt/positional), D156        | `7a9045c4` | +10 Act 1 AI |
| #146 | 5     | `claude/cycle-5-damage-pipeline`          | D91, D124                                      | `10c34602` | +15 pipeline |
| #147 | 6     | `claude/cycle-6-stage-b-tail`             | — (10 rows D172-D181 promoted; 3 latent-closed)| `e38c3ec4` | +13 Stage B (9 `#[ignore]` + 3 green + 1 closed) |
| #148 | 7     | `claude/cycle-7-final-audit`              | audit + doc drift fixes (D91/D124/D162-D165/D179) | this commit | 2328 total |

## Net register deltas

- **Rows closed across 7 cycles:** 17
  - Cycle 2: D162, D163, D164, D165 (4 P0 determinism)
  - Cycle 3: D70, D89, D111 (3 P0 dispatch)
  - Cycle 4a: D143, D144 (2 P0 boss path)
  - Cycle 4b: D118 subset, D152, D154, D155 (alt/positional), D156 (5 P0 enemy AI)
  - Cycle 4c: D140 (1 P0 WrithingMass)
  - Cycle 5: D91, D124 (2 P0 damage pipeline)
  - Cycle 7: D179 (latent-correct reclassification)
- **Rows promoted (new):** 18
  - Cycle 1: D162-D165 (4 F rows) + D166-D169 (4 P1s)
  - Cycle 6: D172-D181 (10 Stage B P1s)
- **Net open-row delta:** +1 (18 promoted − 17 closed)
- **Deferred within closed rows:** D155 BOLT/BEAM constant rename (explicit TODO in register, SpireMonitor cascade noted); D144 Donu/Deca/TimeEater A4/A9 HP-gate conflation (follow-up flagged)

## Per-audit findings

### Audit 1 — F1-F4 determinism (`audit-1-determinism.md`)

**Code: CLEAN.** All four merge-blockers remediated in Cycle 2 `f30066fe`:
- `CombatSnapshotV1.ai_rng_seed0/seed1/counter` at `training_contract.rs:575-579`, restored at `:839-841`.
- `EnemySnapshotV1.move_history: Vec<i32>` at `:528`, restored at `:745` BEFORE `set_move` at `:778-782` (order-critical — `set_move` doesn't touch move_history, so the Vec must be assigned first).
- `search.rs::combat_state_hash` at `:1095-1096` hashes both `engine.rng.state_tuple()` and `engine.ai_rng.state_tuple()` (exactly 2 occurrences required).
- `test_snapshot_determinism.rs` asserts twin-engine parity on `move_id` (L104), `ai_rng.state_tuple()` (L114), AND `move_history` (L109) — all three independent assertions present.
- Python `contracts.py` round-trips `move_history`, `ascension`, `ai_rng_seed0/1/counter`.

**Docs: originally REOPEN.** D162-D165 register rows still marked `**open**` despite code closure. **Fixed in this commit** (rows now stamped with `f30066fe` + closure detail).

### Audit 2 — D70/D89/D111 dispatch (`audit-2-dispatch.md`)

**CLEAN.** Path 2 (PowerDef wiring, not helper resurrection):
- `DEF_EQUILIBRIUM` at `powers/defs/turn_end.rs:194` — TurnEnd + `AddStatus(Player, EQUILIBRIUM, Fixed(-1))`, `status_guard` auto-removes at 0.
- `DEF_FASTING` at `powers/defs/turn_start.rs:525-551` — TurnStart + complex hook draining `engine.state.energy` by FASTING stacks.
- DemonForm (L26/32), NoxiousFumes (L53/59), Brutality (L83/89) retriggered from `TurnStart` → `TurnStartPostDraw` (D111 closure).
- Dead helpers `process_{start,end}_of_turn` / `process_end_of_round` at `powers/buffs.rs:620,704,752` left in place (zero call sites in `engine.rs`; only exercised in unit tests). D83 remains open as their remaining user.
- Integration tests: `tests/test_powers_dispatch_wired.rs` (6 tests) + 4 unit tests all green.

### Audit 3 — Cycle 4 enemy AI vs Java (`audit-3-enemy-ai.md`)

**CLEAN.** All 8 Cycle-4 rows verified against `decompiled/java-src/.../monsters/**`:
- **D143** CorruptHeart slot-0: `num<50 → BLOOD_SHOTS else ECHO` at `act4.rs:123-133` matches Java `CorruptHeart.java:177-184` `aiRng.randomBoolean()`.
- **D144** Ascension gates at `mod.rs:824-825` use `ascension >= 4` / `>= 19`, not HP thresholds — matches Java L76-84, L92-98.
- **D118 subset**: `create_enemy_with_ascension(id, hp, max_hp, ascension)` signature in place; Nob/Snecko/Spiker/SpireShield/CorruptHeart consume `enemy.ascension` in their dispatchers; WrithingMass/Taskmaster/GremlinFat remainder explicitly scoped to umbrella row 181.
- **D152** GremlinWizard 3-turn cycle via `last_two_moves(GREMLIN_PROTECT)` produces PROTECT/PROTECT/BLAST(25) matching Java `currentCharge==3`.
- **D154** Lagavulin 2:1 via `last_two_moves(LAGA_ATTACK) → SIPHON` mathematically equivalent to Java's `debuffTurnCount<2` counter (L209-223).
- **D155** Sentry alternation closed; BOLT/BEAM constant-rename deferred with explicit register TODO.
- **D156** `sentry_fix_first_moves` called from `engine.rs:246` AFTER enemy vector populated; safe on 0/1/3 Sentry combats.
- **D140** WrithingMass Reactive wired in `combat_hooks.rs:582-585` behind `hp > 0` guard.

### Audit 4 — Cycle 5 damage pipeline (`audit-4-damage-pipeline.md`)

**CLEAN.** Three canonical entry points at `engine.rs:2706-2779`:
- `apply_damage_to_player` — NORMAL damage (Wrath / Vuln / Intangible / Block / Torii / Tungsten).
- `apply_hp_loss_to_player` — HP_LOSS (bypass block; Intangible cap; -1 Tungsten).
- `apply_hp_loss_to_enemy` — HP_LOSS + boss-hook bookkeeping via `record_enemy_hp_damage`.

Brutality `Target::Player` at `effects/runtime.rs:1719-1721` → `apply_hp_loss_to_player`.
Pressure Points / TriggerMarks at `effects/interpreter.rs:631` → `apply_hp_loss_to_enemy`.

**Bypass hunt:** 12 raw `.hp -=` sites, all categorized. 3 inside canonical entries. 9 in tick/reactive paths that precompute Intangible + Tungsten locally (Burn/Regret/Pain/Poison/Thorns/Flame Barrier/Static Discharge/potions). **Zero uncategorized bypasses.**

**Tests:** `test_damage_pipeline_routing` 15/15 pass; damage/intangible/vulnerable/wrath/torii/tungsten suites all green (322 tests, 0 fail).

**Docs:** Originally flagged D91/D124 as still `**open**` in register; **fixed in this commit**.

### Audit 5 — Test-suite drift Cycles 2-6 (`audit-5-test-suite.md`)

**Originally REOPEN.** Test suite 2327/0/10 ignored — exact baseline.

**Clean areas:** no new weak asserts introduced Cycles 2-6 (12 pre-existing patterns preserved); no duplicate boss coverage (comments at `test_enemies.rs:339` and `:438` intact); no fresh-RNG leaks (51 `StsRandom::new(0)` confined to `test_enemies.rs`/`test_bosses.rs`).

**Blocker: D179 silently closed.** `d179_plated_armor_not_decremented_by_thorns_or_hp_loss` passed under `--ignored`. Cycle 7 investigation confirmed the test asserts correct Rust behavior: `combat_hooks.rs:186-191` only decrements Plated inside `do_enemy_turns` after an enemy ATTACK intent (NORMAL damage path) — matching Java's `wasHPLost` gate. HP_LOSS ticks use `engine.player_lose_hp(...)` directly, bypassing this path. Enemy-side THORNS retaliation is not yet implemented in Rust (`get_thorns_damage` at `powers/buffs.rs:252` is dead code) — that's a **separate gap** not covered by D179. **Fixed in this commit**: D179 reclassified as latent-correct, test unignored, register row stamped.

**Informational (out of scope):** 43 orphan `.rs` files in `packages/engine-rs/src/tests/` not registered in `mod.rs`. Pre-dates Cycle 1 (commits `03611329`, `fbbf84f4`). Recommend follow-up; do NOT gate this merge.

### Audit 6 — Stage B tail bitrot (`audit-6-stage-b-crossref.md`)

**CLEAN.** 181 register rows (D1-D181), 136 open / 4 deferred / 2 partial-fix / 39 closed.

- D155 deferred TODO is present, worded clearly, cascade sites listed.
- ~230 un-promoted Stage B findings continue to live in per-area reports by design — bitrot risk low.
- Two stale-status rows flagged (D91, D124) — **fixed in this commit**.

**Top-8 candidates for Cycle 8+ Watcher A0 promotion:**
1. WaveOfTheHand early-clear (WS12) — P1
2. Dead-dispatch umbrella for WS7/WS8/WS9/WS10 (LiveForever/AngelForm, WrathNextTurn, EndTurnDeath, NoSkills) — P1 structural
3. Cultist Strength ascension (E3A1) — P1
4. GremlinNob Bellow FRAIL A17+ (E15A1) — P1
5. Hexaghost Inferno A19+ scaling (E6A1) — P1
6. SwiftPotion `Duplication` consumer missing (PT5) — P1
7. Cunning Potion emits Shiv not Shiv+ (PT4) — P2
8. Thorns retaliation Vulnerable/Strength layering (DM2; distinct from D170) — P1

### Audit 7 — combat_hooks integrity (`audit-7-combat-hooks.md`)

**CLEAN.**
- **Debuff double-apply audit:** 3 debuff handlers (Weak/Vuln/Frail at `combat_hooks.rs:287-295`) use `apply_debuff_from_enemy` exactly once per arm with no subsequent duplicate mutation. Non-debuff effects (Strength/Ritual/Constrict/Hex) use direct `add_status`/`set_status` — correctly out of justApplied scope.
- **WrithingMass arm** (`combat_hooks.rs:574-586`): guarded on `enemy.hp > 0`, calls `writhing_mass_reactive_reroll(enemy_idx)` only, no cross-index mutation.
- **`sentry_fix_first_moves`:** called at `engine.rs:246` inside `start_combat` AFTER enemy vector populated; only mutates `enemy.id == "Sentry"` with empty `move_history`; panic-safe on 0/1/3 Sentries.
- **New PowerDefs:** `DEF_EQUILIBRIUM` + `DEF_FASTING` registered in `ALL_POWER_DEFS` + `RUNTIME_PLAYER_POWER_DEFS` (correct player-turn dispatch via `effects::runtime`, not `combat_hooks`).
- **New damage entry points:** all three emit `OnPlayerHpLoss` / dispatch `combat_hooks::on_enemy_damaged` correctly; Thorns / Flame Barrier remain inline per-hit in `execute_enemy_move:240-264` as the correct scoping (fires only on enemy attacks).
- **Tests:** 5/5 `test_combat_hooks_integration` pass, 2/2 intent tests pass, 0 failures.

## Post-remediation verification

```
./scripts/test_engine_rs.sh check --lib                        → OK
./scripts/test_engine_rs.sh test --lib -- --nocapture          → 2328 pass / 0 fail / 9 ignored
uv run pytest tests/training -q                                 → 42 pass / 0 fail
```

## Ship / block decision

**Ship.** All P0 merge-blockers from 2026-04-21 triage are closed. Register is current. Remaining
18 net-open rows are P1/P2 work slated for Cycles 8+. Recommend:

1. Merge PR #138 (audit + triage docs).
2. Merge PRs #139-#147 in stack order (PR #139 base = `codex/training-rebuild`, PR #140 base = PR #139, …, PR #147 base = PR #146).
3. Open Cycle 8 branch off #147 tip with the 8 candidates from Audit 6 pre-ranked.

## Follow-up register (Cycle 8+)

- D83 (dead helpers `process_*_of_turn` — decide: delete or adopt)
- D155 BOLT/BEAM constant rename (SpireMonitor label cascade)
- D144 Donu/Deca/TimeEater A4/A9 HP-gate conflation (umbrella with D144 scope)
- Enemy-side THORNS retaliation against player attacks (separate gap discovered in D179 investigation; register as new D-# when Cycle 8 plans enemy-thorns wiring)
- 43 orphan `.rs` files in `packages/engine-rs/src/tests/` unregistered in `mod.rs` (pre-exists Cycle 1; hygiene cleanup)
- 186 clippy warnings (D121)
- 8 Watcher-A0 promotions listed in Audit 6 §Top-8
