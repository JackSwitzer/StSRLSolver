# Audit 3 — Cycle 4 enemy-AI closures vs Java

**Status:** CLEAN
**Rows audited:** D118 (subset), D140, D143, D144, D152, D154, D155 (alternation half), D156
**Rust test count:** 210/210 enemies + 9/9 ai_rng_parity + 37/37 bosses + 5/5 combat_hooks — zero failures across all four suites

## Per-row verification

- **D143 CorruptHeart slot-0 (`act4.rs:123-133`):** CLEAN. Java `CorruptHeart.java:177-184` — `case 0: if (aiRng.randomBoolean()) BLOOD_SHOTS else ECHO`. Rust gates `num < 50 -> BLOOD_SHOTS else ECHO` and sets `bloodHitCount`/`echoDmg` from status, matching Java's `damage.get(1)/bloodHitCount` and `damage.get(0)`. Slot 1 deterministic anti-repeat and slot 2 escalating buff both match Java L186-197.

- **D144 ascension-gate (`mod.rs:809-830`):** CLEAN. Java `CorruptHeart.java:76-84` — `ascensionLevel >= 4 -> damage 45 / bloodHitCount 15`, else 40/12. Java L92-98 — `ascensionLevel >= 19 -> invincibleAmt -= 100, beatAmount += 1` (net 200 invincible, 2 beat). Rust: `if ascension >= 4 { (15, 45) } else { (12, 40) }`, `if ascension >= 19 { (200, 2) } else { (300, 1) }`. HP gates are gone — scaling now correctly keys on run ascension, not Heart HP (D144 root cause resolved). HP pool (750 vs 800) stays caller-supplied per Java L71-74 and is documented as such.

- **D118 subset (`mod.rs:412`):** CLEAN. `create_enemy_with_ascension(id, hp, max_hp, ascension)` exists, sets `enemy.ascension = ascension` at line 419, and production callers use it (`run.rs:1245`; tests use it for all five enemies). Construction-time ascension consumers: GremlinNob ENRAGE A2+ (mod.rs:498), SpireShield BASH A3+ (mod.rs:797). Dispatcher-time via `enemy.ascension`: Snecko bite/vuln A2+/A17+ (act2.rs:296-297), Spiker buff A17+ (act3.rs:85), SpireShield bash A3+ (act4.rs:32), CorruptHeart full pipeline (mod.rs:824-825). All 5 enemies threaded. WrithingMass/Taskmaster/GremlinFat remain scoped to this umbrella as `**open**` tail — register row 181 states this explicitly.

- **D152 GremlinWizard (`act1.rs:307-326`):** CLEAN. Java L142-144 `getMove` always sets CHARGE; Java L66-96 takeTurn only sets ATTACK when currentCharge reaches 3 (init 1). Rust: opener = GREMLIN_PROTECT (mod.rs:489); `last_two_moves(GREMLIN_PROTECT)` is the 3-turn gate. Sequence: PROTECT, PROTECT, ULTIMATE_BLAST(25), PROTECT, ... matches Java's MAGIC_DAMAGE=25 at L39. Tests `gremlin_wizard_{turns_1_2_charge, turn_3_ultimate_blast, full_cycle_repeats}` cover it.

- **D154 Lagavulin 2:1 (`act1.rs:352-382`):** CLEAN. Java L209-223 — awake branch: `debuffTurnCount < 2 && !lastTwoMoves(STRONG_ATK=3)` → ATTACK, else DEBUFF(SIPHON). Counter increments on ATTACK, resets on DEBUFF → cycle ATTACK, ATTACK, SIPHON. Rust `last_two_moves(LAGA_ATTACK) -> SIPHON else ATTACK` is mathematically equivalent because the counter only >=2 precisely when the two most recent moves were both ATTACK. SIPHON adds STR/DEX -1 (matches Java L119-120 using `this.debuff`). Tests `lagavulin_attack_attack_debuff_cycle` + `lagavulin_siphon_carries_str_and_dex_debuff` GREEN.

- **D155 Sentry alternation (`act1.rs:391-403`):** CLEAN (alternation half). Java L142-146: `lastMove(BEAM) ? BOLT : BEAM`. Rust `last_move(SENTRY_BOLT) ? SENTRY_BEAM : SENTRY_BOLT` is label-swapped but semantically equivalent given the pre-existing label inversion. Register row 229 explicitly carries the outstanding substructure half (BOLT/BEAM constant rename + splitting the 9-damage-plus-DAZE combo) as a follow-up — marked `**closed (alternation half)**` not fully closed, which is correct.

- **D156 Sentry positional (`act1.rs:424-443`, `engine.rs:246`):** CLEAN. Helper `sentry_fix_first_moves` iterates `state.enemies`, skips non-Sentries + any Sentry with `move_history.is_empty()==false`, then assigns BEAM (idx even) or BOLT (idx odd). Java Sentry.java:132-141 uses `monsters.lastIndexOf(this) % 2`: even → BOLT (Java-semantics = Rust SENTRY_BEAM), odd → BEAM (Java-semantics = Rust SENTRY_BOLT). Grep confirms exactly 2 refs in `src/`: definition at `act1.rs:424` + call site at `engine.rs:246` (plus a doc comment at `run.rs:1250` noting the hard-coded stagger was obsoleted). Positional openers cover every encounter shape, not just the 3-Sentry triple. Tests `sentry_position_{0,1,2}_opens_{beam,bolt,beam}` GREEN.

- **D140 WrithingMass Reactive (`combat_hooks.rs:574-586`, `act3.rs:188-215`):** CLEAN. Dispatch arm `"WrithingMass"` in `on_enemy_damaged` guards on `enemy.hp > 0` and calls `enemies::writhing_mass_reactive_reroll(&mut state.enemies[enemy_idx])`. Java `WrithingMass.damage()` re-invokes `rollMove`/`getMove` on every damage instance; Rust now cycles intent on each non-lethal hit. Grep confirms 2 refs in `src/` (definition at `act3.rs:188` + call site at `combat_hooks.rs:582`). Test `writhing_mass_reactive_reroll_fires_on_damage` GREEN.

## Test run (four suites, zero failures)

- `./scripts/test_engine_rs.sh test --lib enemies` → 210 passed / 0 failed / 2127 filtered
- `./scripts/test_engine_rs.sh test --lib ai_rng_parity` → 9 passed / 0 failed / 2328 filtered
- `./scripts/test_engine_rs.sh test --lib bosses` → 37 passed / 0 failed / 2300 filtered
- `./scripts/test_engine_rs.sh test --lib combat_hooks` → 5 passed / 0 failed / 2332 filtered

## Register hygiene

All 8 rows in `docs/work_units/parity-deviations-register.md` carry `**closed 2026-04-21**` status with commit SHAs (4a: `a5fc04b4`; 4b subset D118: `e5108bdf`; 4b-rest: `7a9045c4`; 4c D140: `69c0da9a` cited in row, Cycle 4c / PR #141). D155 explicitly marked `**closed (alternation half) 2026-04-21**` with outstanding substructure half (constant rename / damage-daze split) documented — correct per PR report.

## Recommendation

**Ship.** All 8 rows verified against Java decompile; all four targeted test suites pass; register closures are explicit and SHA-stamped; D155 half-close is honestly disclosed.
