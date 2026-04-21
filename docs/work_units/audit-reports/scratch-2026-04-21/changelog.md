# PR #138 Changelog — `claude/parity-audit-fleet-2026-04-21` vs `main`

**Auditor:** read-only audit subagent, 2026-04-21
**Commits:** 7 (`55e77c89` .. `d16cdba2`)
**Diff:** 24 files, +4170 / -547

## Summary table (fixed vs deferred vs added)

| Subsystem | Fixed (behavior change) | Deferred (registered only) | Added (new code / infra) | Tests | Docs |
|---|---|---|---|---|---|
| Enemy AI (act1-act4, mod.rs) | D1 core branches (~40 enemies re-wired to consume num); D129 Jaw Worm branch swap; D130 Collector off-by-one; Spiker double-Thorns (review fix) | D131, D132, D133-D157 (24 dispatcher bugs flagged), D136/D140/D143/D145/D152/D154/D155 P0/P1 | `roll_next_move_with_num`, `last_move`, `last_two_moves` helpers; `writhing_mass_reactive_reroll` (orphan) | ~80 pattern tests re-targeted with explicit `num`; D158 tautology removed | Inline deferred-subroll comments in each dispatcher |
| Combat hooks / damage | — | D124 (Pressure Points bypass), D140 (WrithingMass Reactive un-wired), D112 (deal_damage_to_player bypass), D91 effect-DSL damage | — | — | — |
| Powers / relics / cards | — | D88 HolyWater, D89 Fasting, D94 Malleable, D123 Deva Form, D125 Swivel, D126 Miracle retain, D127 Collect+, D128 Signature Move, D122 Crippling Cloud ID, D93..D121 | — | — | — |
| Training infra (scripts/, packages/training/) | `scripts/training.sh` launch no longer appends onto stale `logs/active`; pid regex guard; `BOSS_SOLVE_VALIDATION_THRESHOLD`, `COLLECTION_WORKER_COUNT`, `ROOM_KIND_CORPUS_WEIGHTS` lifted out of magic numbers; synthetic corpus now weighted 1/2/3 (hallway/elite/boss) | 12-worker manifest claim (resolved by telling truth, not by adding workers); hardcoded ascension→HP conflation upstream | `archive_active()`, `smoke` subcommand, `--with-smoke` launch flag, `_weighted_encounter_pool()` (fixed-seed shuffle) | — | — |
| Audit docs | — | — | 11 per-area reports under `docs/work_units/audit-reports/` (~1919 LOC); register extended from ~87 rows to D159 | — | Register rows D88-D159 added |

## Per-commit changelog

### 1. `55e77c89` — audit: Stage B fleet audit + Stage C register consolidation

- **Intent:** produce 11 parallel-subagent parity reports against `decompiled/java-src` and promote top findings into the register.
- **Files:** all docs. `docs/work_units/audit-reports/{damage-engine-flow,enemies-act1,enemies-act2-act3-act4,other-class-cards,potions,powers-buffs-debuffs,relics,run-map-events-shop,tests-types-api,watcher-cards,watcher-stances-scry-mantra}.md` (+1796 LOC); `docs/work_units/parity-deviations-register.md` (+123 / -2).
- **Fixes:** none. Pure documentation commit.
- **Adds:** 11 audit report files.
- **Registers:** ~302 total findings, 41 promoted to D88-D128. Headline: D88 (HolyWater→SafetyPin), D89 (Fasting never deducts energy), D94 (Malleable no-reset), D112 (`deal_damage_to_player` bypass), D124 (Pressure Points bypass), D126/D127 (Miracle+Collect compound).

### 2. `9984bd86` — parity: Stage D — enemy AI getMove bodies consume aiRng (D1 Phase 2)

- **Intent:** thread `num` from `aiRng.random(99)` through per-enemy dispatchers, restoring Java's `lastMove`/`lastTwoMoves` anti-repeat guards.
- **Files (9):** `enemies/act1.rs` (+204/-…), `enemies/act2.rs`, `enemies/act3.rs`, `enemies/act4.rs`, `enemies/mod.rs` (+925), `tests/test_bosses.rs`, `tests/test_enemies.rs`, `tests/test_enemy_ai.rs`, register.
- **Fixes (behavior change):**
  - `enemies/mod.rs:829` new `roll_next_move_with_num(enemy, num)` entry point. `roll_next_move` (production) now draws `num = ai_rng.random(99)` then delegates. Previously enemies consumed no RNG (pre-Phase-1 baseline).
  - Act 1 rewrites: RedLouse, GreenLouse, SlaverBlue, SlaverRed, AcidSlime_{S,M}, GremlinNob, Cultist, JawWorm, FungiBeast (branching on `num` — see caveats per D146/D149/D150/D151 below).
  - Act 2: Chosen, ShelledParasite, Byrd, SnakePlant, Centurion, Mystic, BookOfStabbing, Snecko, BronzeOrb, Champ (incl. phase-2 Execute + turn-4 Taunt), Collector (turns `== 4` gate — later corrected in `d29dac03`).
  - Act 3: Darkling, OrbWalker, Spiker (roll-time Thorns bump — later removed in `d16cdba2`), Repulsor, Exploder, WrithingMass + orphan `writhing_mass_reactive_reroll` (never wired to combat_hooks — tracked as D140), SpireGrowth, Maw, GiantHead, Nemesis, Reptomancer, AwakenedOne P1/P2, TimeEater.
  - Act 4: SpireShield cycle, SpireSpear anti-repeat.
- **Adds:** `last_move`, `last_two_moves` pub(crate) helpers (`mod.rs:917/921`).
- **Registers:** D1 marked **fixed** (core branch logic) + deferred-sub-rolls noted inline. Register row touched (+1/-1).
- **Test updates (not fixes):** `test_enemies.rs`, `test_enemy_ai.rs`, `test_bosses.rs` pattern expectations rewritten to call `roll_next_move_with_num` with specific nums.

**Caveat vs commit title:** the message says "rewrote per-enemy dispatchers across Act 1 (10 enemies)". GremlinWizard (`act1.rs:307` `roll_gremlin_wizard(enemy)` — no num param, 2-turn PROTECT/ATTACK cycle), Lagavulin (`roll_lagavulin(enemy)`), Sentry (`roll_sentry(enemy)`), Guardian, Hexaghost, SlimeBoss all **kept their original non-num bodies**. Stage G later registered these as P0/P1 drift (D152 Wizard 3-turn, D154 Lagavulin 2:1, D155/D156 Sentry). FungiBeast and SpikeSlime_{M,L} accept `_num` but discard it (D146/D147/D148). BanditLeader/Bear per-line 319/307 also discard `_num` (D134/D135).

### 3. `38d672d8` — ops: Stage E — training.sh archive + smoke preflight

- **Intent:** prevent overnight relaunches from appending onto stale `logs/active`; add pre-run smoke.
- **Files:** `scripts/training.sh` (+103/-32).
- **Fixes (behavior change):**
  - `archive_active()` at `training.sh:6-33`: refuses if pid still running, else moves `logs/active` → `logs/runs/<utc-stamp>`. Called automatically from `launch` (`training.sh:68`).
  - `smoke` subcommand (`training.sh:40`) runs `run-phase1-puct-overnight --target-cases 24 --collection-passes 1 --epochs 1` into `logs/smoke/<utc-stamp>`, exits nonzero on failure.
  - `launch --with-smoke` flag (`training.sh:61`) blocks launch on red preflight.
- **Adds:** three new subcommands. Bash-only, additive. No Python infra change.
- **Registers:** none. Commit body calls out deferred Stage-E items (hardcoded seed-validation threshold, 12-worker manifest claim, Watcher-boss weighting) — fixed in next commit.

### 4. `f0965a04` — training: Stage E — surface hidden constants + weight corpus

- **Intent:** lift three magic numbers in Phase 1 PUCT overnight; weight synthetic encounters toward bosses/elites.
- **Files:** `packages/training/cli.py` (+1/-1 import + usage), `packages/training/stage2_pipeline.py` (+44/-3).
- **Fixes (behavior change):**
  - `stage2_pipeline.py:43` `COLLECTION_WORKER_COUNT = 1`; `cli.py:701` `worker_count=12` → `worker_count=COLLECTION_WORKER_COUNT`. Manifest now reports truth.
  - `stage2_pipeline.py:49` `BOSS_SOLVE_VALIDATION_THRESHOLD = 0.5` replaces inline `>= 0.5` at `stage2_pipeline.py:777`.
  - `stage2_pipeline.py:56` `ROOM_KIND_CORPUS_WEIGHTS = {hallway:1, elite:2, boss:3}` drives new `_weighted_encounter_pool()` (`stage2_pipeline.py:315`) used by `_build_synthetic_snapshot_cases` (`stage2_pipeline.py:327`).
  - `_ENCOUNTER_POOL_SHUFFLE_SEED = 20260421` (`stage2_pipeline.py:308`) keeps corpus generation reproducible across small `target_cases`.
- **Adds:** `_weighted_encounter_pool()`, four module constants.
- **Registers:** none.

### 5. `d29dac03` — parity: Stage F — Jaw Worm + Collector fixes

- **Intent:** two pre-existing bugs whose Rust test coverage encoded the bug as "Java parity".
- **Files (8):** `enemies/act1.rs` (+51/-…), `enemies/act2.rs` (+21/-…), `enemies/mod.rs` (+43/-…), `tests/test_ai_rng_parity.rs` (+84/-…), `tests/test_bosses.rs`, `tests/test_enemies.rs`, `tests/test_enemy_ai.rs`, register.
- **Fixes (behavior change):**
  - **D129 Jaw Worm BELLOW/THRASH swap:** `act1.rs:11-52` rewrite. Before: `num<25→CHOMP / 25-54→BELLOW / else→THRASH`. After (matches `JawWorm.java:146-181`): `num<25→CHOMP / 25-54→THRASH / 55-99→BELLOW`. Dropped 2nd-aiRng sub-rolls fall through to dominant branch (documented at `act1.rs:26-28`; tracked as D131).
  - **D130 Collector MegaDebuff off-by-one:** `act2.rs:492` `turns == 4 && !ult_used` → `turns >= 3 && !ult_used`. Detailed Java post-increment reasoning at `act2.rs:486-491`.
- **Registers:** D129 **closed**, D130 **closed**, D131 **deferred** (sub-roll 2nd RNG), D132 **open** (Byrd grounded Fly-Up synthesis).
- **Test rewrites (encoded bug as parity → now assert Java):** `test_ai_rng_parity.rs` Jaw Worm tests renamed `*_defaults_to_*`, distribution counts 25/30/45; `test_enemy_ai.rs` / `test_enemies.rs` / `mod.rs` swap `num=30→THRASH`, `num=80→BELLOW`; Collector `MegaDebuff on call 3` (was call 4); `test_collector_buff_branch` seeds post-MegaDebuff history.

### 6. `0b192869` — audit: Stage G — 27 findings + `|| true` tautology fix

- **Intent:** 4 parallel audit subagents over Stage F baseline; register 24 AI-dispatcher bugs + 1 tautology + consolidation finding; fix only the tautology.
- **Files (2):** `docs/work_units/parity-deviations-register.md` (+38), `packages/engine-rs/src/tests/test_enemies.rs` (+5/-5).
- **Fixes (behavior change):**
  - **D158 tautology:** `test_enemies.rs:603-607` removed `heart_has_invincible` whose assertion was `assert!(... || true)` — the `|| true` made the check unconditional. Breadcrumb comment cites `test_bosses.rs::corrupt_heart_initial_invincible_matches_java` (L509-525) for equivalent coverage.
- **Registers (D133-D159, all except D158 open/deferred):**
  - P0: D143 (CorruptHeart slot 0 det. Blood Shots), D144 (Heart A4-A8 scaling), D140 (WrithingMass Reactive un-wired), D152 (GremlinWizard 2-turn vs Java 3), D154 (Lagavulin 1:1 vs 2:1), D155 (Sentry BOLT/BEAM swap).
  - P1: D146/D147/D148 (FungiBeast + SpikeSlime_{M,L} discard num), D141 (Transient damage direction reversed), D142 (Exploder skips UNKNOWN turn-2), D133 (BronzeAutomaton HyperBeam turn-5-vs-6), D136 (GremlinLeader ignores aliveCount).
  - P2: D134/D135 Bandit re-emission, D137/D138 Champ Anger/Defensive, D139 Snecko threshold, D145 Darkling turn-1, D149 AcidSlime_S threshold, D150 Louse single-vs-double, D151 SlaverBlue extra guard, D153 GremlinTsundere empty arm, D156 Sentry first-move positional, D157 Looter stream off-by-one.
  - D159 **deferred** consolidation: `test_enemies.rs` (647 LOC, 83 tests) fully superseded; ~45 duplicate tests in `mod.rs:946-1693`.
- **Test count:** 2291 → 2290 (one removed tautology).

### 7. `d16cdba2` — address review findings

- **Intent:** reviewer-requested corrections from Stage D residue and training.sh hardening.
- **Files (3):** `enemies/act3.rs` (+3/-2), `tests/test_enemy_ai.rs` (+2/-2), `scripts/training.sh` (+6/-1 net).
- **Fixes (behavior change):**
  - **Spiker double-Thorns:** `act3.rs:84` removed roll-time `enemy.entity.set_status(sid::THORNS, thorns + 2)`. `combat_hooks` already applies +2 via `mfx::THORNS` at intent-execute, so roll-time bump was `+4/turn` instead of Java's `+2/turn`. The Stage D commit had introduced this bug.
  - **Spiker test `test_enemy_ai.rs:691`:** assertion flipped from `THORNS == 5` to `THORNS == 3` (init). Old 5 was encoding the bug as parity.
  - **training.sh pid regex guard `training.sh:20`:** `if [[ -n "$prev_pid" ]]` → `if [[ "$prev_pid" =~ ^[0-9]+$ ]]`. Protects against malformed pid_file probing an unrelated process.
- **Registers:** none.

## Subsystem roll-up

### Enemy AI (act1.rs, act2.rs, act3.rs, act4.rs, mod.rs) — largest surface

- **Before branch:** `roll_next_move(enemy)` — no `ai_rng` threading; most dispatchers were fully deterministic sticky cycles; `last_move`/`last_two_moves` guards missing.
- **After branch:**
  - New `roll_next_move_with_num(enemy, num)` entry in `mod.rs:829` — pushes move_history, clears move_effects, dispatches on `enemy.id`. Production `roll_next_move` draws `num = ai_rng.random(99)`.
  - `last_move` / `last_two_moves` helpers at `mod.rs:917/921`.
  - Roughly 40 enemies now branch on `num` for their primary split (CHOMP/THRASH/BELLOW for Jaw Worm, FLAIL/BOOST for BronzeAutomaton, phase-gated logic for Champ/Collector/AwakenedOne/TimeEater).
- **Behavior changes beyond D1 threading:**
  - `d29dac03` Jaw Worm ordering (D129 closed), Collector `turns >= 3` (D130 closed).
  - `d16cdba2` Spiker: removed roll-time Thorns bump (was introduced in `9984bd86` — this reverts the double-apply).
- **Not-fixed but touched:** WrithingMass `writhing_mass_reactive_reroll` (`act3.rs:186`) exists as a pub helper but is called only by test `test_enemy_ai.rs:720` — never from combat_hooks. Registered as D140 P0.
- **Not-rewritten:** GremlinWizard (`act1.rs:307`, no num param), Lagavulin, Sentry, Guardian, Hexaghost, SlimeBoss, BanditLeader/Bear (all `_num` discarded). All flagged in Stage G D133-D157.

### Tests (test_enemies.rs, test_enemy_ai.rs, test_bosses.rs, test_ai_rng_parity.rs)

- **Added/updated in `9984bd86`:** ~80 pattern tests retargeted to `roll_next_move_with_num(num)` so specific Java branches are selected deterministically. Champ turn-4 Taunt, TimeEater Haste+HeadSlam+Ripple cycles updated.
- **Updated in `d29dac03`:** Jaw Worm distribution (25/30/45), `num=30→THRASH`/`num=80→BELLOW` swaps, Collector MegaDebuff on call 3.
- **Removed in `0b192869`:** `heart_has_invincible` (D158 closed, test_enemies.rs:603-607 replaced with breadcrumb comment).
- **Updated in `d16cdba2`:** Spiker test expects `sid::THORNS == 3` (post-roll, pre-execute).

### Combat hooks / damage — UNCHANGED

No code changes to `combat_hooks.rs`, `effects/interpreter.rs`, or damage pipeline. All audit findings (D112, D124, D140 wiring) remain open/deferred.

### Training infra (scripts/, packages/training/)

- **`scripts/training.sh`** — rewritten from single `launch` branch into `case launch|archive|smoke` dispatcher. Auto-archive on launch; `--with-smoke` preflight. pid regex guard added in `d16cdba2`.
- **`packages/training/stage2_pipeline.py`** — three named constants added, `_weighted_encounter_pool()` with fixed-seed shuffle, `build_seed_validation_report` now references `BOSS_SOLVE_VALIDATION_THRESHOLD`.
- **`packages/training/cli.py`** — imports `COLLECTION_WORKER_COUNT`, passes to PuctCollectionRecord manifest.

### Docs

- 11 Stage B reports (`audit-reports/*.md`) — 1919 LOC.
- Register grew from ~87 rows to D159; fleet-audit index prepended (line ~197 "Stage G audit fleet — 2026-04-21" section).
- No new user-facing documentation files.

## Register cross-check

| Dn | Register status | Actual code state (HEAD `d16cdba2`) | Match? | Notes |
|---|---|---|---|---|
| D1 | **fixed** for core branch logic; deferred sub-rolls + engine-context branches noted | `roll_next_move` threads `num`; ~40 enemies branch on it; ~10 enemies still discard `_num` (GremlinWizard, Lagavulin, Sentry, Guardian, Hexaghost, SlimeBoss, BanditLeader/Bear, FungiBeast, SpikeSlime_{M,L}) | **partial** | Register says "core branch logic fixed"; ~10 enemies not core-fixed. Register narrative does acknowledge "Remaining gaps… deferred secondary-roll retries, aliveCount-dependent… ascension-gated thresholds" but doesn't list the plain-ignored-num cases. Stage G D146/D147/D148/D152/D153/D154/D155/D156 register these properly. |
| D129 | **closed** (Stage F) | `act1.rs:11-52` matches Java `num<25→CHOMP / <55→THRASH / else→BELLOW`. Tests assert Java distribution. | yes | Clean close. |
| D130 | **closed** (Stage F) | `act2.rs:492` `turns >= 3 && !ult_used`. Tests expect MegaDebuff on call 3. | yes | Clean close. |
| D131 | **deferred** (sub-roll 2nd aiRng) | `act1.rs:30-50` picks dominant branch with comment "Deferred sub-roll". | yes | Matches narrative. |
| D132 | **open** (Byrd grounded Fly-Up) | `act2.rs` roll_byrd synthesis retained (no fix in this PR). | yes | Correctly open. |
| D133-D157 | **open** (Stage G) | Each entry's Rust ref confirmed unchanged. Spot-checked D140 (`writhing_mass_reactive_reroll` only called from test), D146 (`roll_fungi_beast(enemy, _num)` discards num), D152 (`roll_gremlin_wizard(enemy)` 2-turn), D153 (`mod.rs:853` `/* Does nothing each turn */`). | yes | 25 rows all correctly "open". |
| D158 | **closed** (Stage G) | `test_enemies.rs:603-607` replaced with breadcrumb comment. `heart_has_invincible` gone. | yes | Clean close. |
| D159 | **deferred** (consolidation PR) | test_enemies.rs still 647 LOC; mod.rs:946-1693 duplicate block intact. | yes | Matches. |

No register row in this PR's range claims a closure that the code does not back up. The only soft mismatch is D1's "fixed for core branch logic" — defensible given the narrative's listed deferrals, but the audit reader should know ~10 Act-1 enemies still discard `_num` (now properly tracked via Stage G rows).

## Meta concerns

1. **Stage D commit title overstates coverage.** `9984bd86` claims "Act 1 (10 enemies)… num-based branches match Java getMove exactly". In reality FungiBeast, GremlinWizard, Lagavulin, Sentry, Guardian, Hexaghost, SlimeBoss, BanditLeader/Bear still discard `_num` or use non-Java-parity cycles. Stage G (`0b192869`) correctly registers the gaps as D146-D157, but the Stage D title alone would mislead a reviewer. The D1 register entry mitigates this by listing deferrals.

2. **Stage D introduced a bug fixed in `d16cdba2`.** Spiker roll-time `sid::THORNS += 2` was added in `9984bd86` (`act3.rs`) and double-applied with `combat_hooks`' intent-execute `mfx::THORNS`. Reviewer caught it; the Stage D test at `test_enemy_ai.rs:691` was asserting `THORNS == 5` (2 init + 2 roll + undelivered 2 execute == wrong accounting). This is a real parity regression introduced by this PR and cleaned up in the same PR — worth calling out because the Stage D test assertion was encoding the freshly-introduced bug as parity.

3. **D140 / `writhing_mass_reactive_reroll` shape.** The helper was added in Stage D as a `pub fn` with a detailed comment, but the combat_hooks wiring that would make it behave like Java was never written. It exists as dead production code with exactly one test consumer (`test_enemy_ai.rs:720`). Stage G correctly flags this as P0, but the Stage D commit could be read as "WrithingMass reactive re-roll implemented" when in fact it is inert. `mod.rs:934` re-exports it, which makes it reachable from outside `enemies::act3` — the test path is currently the only consumer.

4. **Register row D1 claims "for core branch logic"** — subjective but the reader should know that ~10 visibly-non-parity dispatchers (GremlinWizard 2-turn cycle in particular is a P0 per Stage G) are considered outside "core" here. Consider tightening the phrasing or linking to D146-D157 from the D1 row.

5. **Stage E "12-worker" fix is a docs fix, not a performance fix.** `COLLECTION_WORKER_COUNT = 1` is the correct manifest value because PUCT collection is actually single-process; the prior `worker_count=12` was a lie. This is a real bug fix in manifest-consumers' eyes, but commit body could be clearer that the answer is "set the constant to 1", not "add workers".

6. **Duplicate / near-duplicate tests pre-exist the PR.** D159 deferred; no action in this PR. Worth flagging because several Stage G findings (D129/D130 pre-Stage-F) were hidden inside tests that encoded the bug as parity — the 35% LOC reduction proposed in D159 would reduce the chance of future hidden drift.

7. **`scripts/training.sh` launcher has no integration test.** The `archive_active` + `smoke` + `launch --with-smoke` logic is bash-only and not exercised by any pytest. This is consistent with "bash-first scripts" preference but worth knowing when reviewing the change.
