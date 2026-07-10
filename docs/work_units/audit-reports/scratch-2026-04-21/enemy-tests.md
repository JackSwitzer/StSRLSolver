# Enemy-test parity audit (PR #138 pre-merge) — 2026-04-21

Read-only audit. Central question: **do Rust enemy tests validate Java
parity, or only that Rust matches itself?** Trigger: the Spiker Thorns
double-apply (test encoded `expect_status(sid::THORNS, 5)` as "parity"
while Java computed 3). Scope: four test files:

- `packages/engine-rs/src/tests/test_enemy_ai.rs` (911 lines)
- `packages/engine-rs/src/tests/test_enemies.rs` (646 lines)
- `packages/engine-rs/src/tests/test_bosses.rs` (554 lines)
- `packages/engine-rs/src/tests/test_ai_rng_parity.rs` (226 lines)

Severity bands: **P0** active bug-encoding / guaranteed silent pass.
**P1** weak assertion that would mask a real bug. **P2** smaller hole
(gap, duplication, unreachable branch).

## Summary

- No remaining true Spiker-shape post-roll-only asserts. The single
  historical instance (`test_enemy_ai.rs:688`) was fixed at commit
  `d16cdba2`; now asserts init value 3 with explanatory comment.
- One P0 latent double-apply risk via `mfx::ARTIFACT`: Donu/Deca init-state
  parity asserts happen *before* any roll so they're fine, but their
  `roll_*` counterparts drive the buff move that emits `mfx::ARTIFACT`
  which `combat_hooks` *adds* on top of the init value. No test currently
  reads `sid::ARTIFACT` after rolling Donu/Deca through enough turns to
  expose a drift, so a second double-apply of the Spiker shape would not
  fail any test.
- Main weakness is in **test_enemies.rs** (older file): weak threshold
  asserts, conditional wraps that silently pass when the branch isn't
  hit, and fresh-RNG-per-call that pins num=0 so half the Java
  branch-space never runs.
- **test_bosses.rs** and **test_ai_rng_parity.rs** are healthy; strong
  Java-citation discipline and proper num-driven branch enumeration.
- **test_enemy_ai.rs** is mostly healthy after the Spiker fix; its
  status asserts are either pre-roll init (pure parity) or
  self-consistency on roll-only counters the enemy writes itself.

## Post-roll-only asserts (Spiker shape — CRITICAL)

No remaining P0 active-bug instances. One **P1** latent pattern:

- **P1** `test_enemy_ai.rs:685-691` (Spiker) — **fixed** at commit
  `d16cdba2`; assert is now `expect_status(&e, sid::THORNS, 3)` after
  one roll, with a comment explaining that combat_hooks applies
  `mfx::THORNS` at intent-execute time. Kept on the P1 list because it
  is the canonical regression site — any future refactor that reverts
  the comment or bumps the expected value must be rejected. No action
  needed if the comment stays.
- **P1** Donu/Deca `sid::ARTIFACT` is never read after a roll. Init
  asserts (`test_bosses.rs:408,426,440,452`) are pre-roll so they're
  parity. But Donu's `DONU_CIRCLE` intent emits `mfx::STRENGTH, 3` (and
  `mfx::ARTIFACT` via the cycle at 3rd buff); if any future code ever
  starts setting `sid::ARTIFACT` inside `roll_donu`, combat_hooks would
  silently double-add and no test would catch it. Gap, not current bug.
- Sites verified **safe** (init-only or self-consistency):
  `test_enemy_ai.rs:121,125,129,193,197,198,349,351,356,375,379,383,
  396,513,518,610,617,621,622,633,637-639,643-644,648-649,768,775,789,
  827,831-832`. The self-consistency statuses (`STAB_COUNT`, `COUNT`,
  `SCYTHE_COOLDOWN`, `MOVE_COUNT`, `SLEEP_TURNS`, `TURN_COUNT`,
  `FIRST_MOVE`, `ATTACK_COUNT`, `STARTING_DMG`, `SHIFTING`,
  `SKEWER_COUNT`) are written by the enemy's own roll code;
  combat_hooks does **not** mirror them via `mfx::*`, so no
  double-apply risk.

## Tautologies / weak asserts

- **P1** `test_enemies.rs:132` `assert!(e.entity.status(sid::CURL_UP) > 0)`
  — RedLouse Java sets exactly 5; any positive passes. Silently hides
  drift to 1, 7, or any non-zero mistake.
- **P1** `test_enemies.rs:157` — GreenLouse CURL_UP same shape.
- **P1** `test_enemies.rs:492` `assert!(e.effect(mfx::VULNERABLE).is_some())`
  — GremlinNob Skull Bash Java = Vuln 2; `is_some()` passes for any
  value including 0-reserved marker. Should be `== Some(2)`.
- **P1** `test_enemies.rs:507` `assert!(e.entity.status(sid::METALLICIZE) >= 8)`
  — Lagavulin A0 is exactly 8; a drift to 9/10 would pass.
- **P1** `test_enemies.rs:527` `assert!(e.move_hits() >= 2)` — Book of
  Stabbing init hits is exactly 2.
- **P1** `test_enemies.rs:535` `assert!(e.move_hits() >= initial_hits, ...)`
  — "should not decrease" is weaker than Java's "count++". Doesn't
  catch a stuck counter.
- **P1** `test_enemies.rs:551,573` `e.move_damage() >= 40` / `>= 45` —
  Nemesis Scythe (45) and Automaton Hyper Beam (45) have exact values;
  threshold check hides an off-by-5 drift.
- **P1** `test_enemies.rs:611` `e.move_hits() >= 12 || e.move_id == HEART_BLOOD_SHOTS`
  — disjunction with the move-id fallback means a wrong-hits Blood
  Shots still passes as long as the id matches. Java is exact 12 hits
  on A0, 15 on A19.
- **P2** `test_enemies.rs:622` `assert_eq!(e.move_damage(), 6)` for an
  unknown enemy id — tests the default-enemy fallback, not parity.
  Acceptable but off-topic for parity audit.

## Self-consistency tests (reads back its own roll writes)

These are valuable as regression smoke but are **not Java parity**.
Flagging so they aren't mistaken for parity coverage.

- **P2** `test_enemy_ai.rs:617,633,648,649,768,775,789,827,831-832` —
  `TURN_COUNT`, `COUNT`, `SCYTHE_COOLDOWN`, `FIRST_MOVE`, `MOVE_COUNT`,
  `SKEWER_COUNT` asserts read counters written by the enemy's own
  roll/init code. They prove our wiring is internally consistent but
  don't prove Java emits the same counter cadence.
- **P2** `test_enemies.rs:102` `SPORE_CLOUD = 2`, `:344` `MODE_SHIFT = 30`,
  `:366-371` `SHARP_HIDE = 3` / `MODE_SHIFT = 40`, `:386` `SHARP_HIDE = 0`,
  `:482` `ENRAGE = 2`, `:589-593` `CURIOSITY = 1` / `PHASE = 1` — init
  state values match Java, but they are written in `create_enemy` in
  `enemies/mod.rs` (lines 476, 480, 531, 536, 600, 657, 737-741, 795,
  800); testing them asserts the constructor, not `getMove`.

## Missing ascension coverage

- **P1** **test_enemies.rs** has **zero** explicit ascension variants.
  Every `create_enemy(... base_hp)` call uses A0 HP; higher ascension
  branches in `act1.rs`, `act2.rs`, `act3.rs`, `act4.rs` that key off
  HP thresholds are untested from this file.
- **P1** **test_enemy_ai.rs** `actN_patterns_match_java` — all rolls use
  A0 HP. A2/A4/A9/A19 branches unique to
  act1.rs (Nob A2 StrengthDown), act2.rs (Champ Anger A4+),
  act3.rs (Nemesis Scythe dmg scaling, AwakenedOne curiosity count)
  are not exercised here. `run_engine_exposes_ascension_hp_tables`
  (line 854) checks HP + enemy-id lookups only — no post-roll move
  values at A20.
- **P2** **test_bosses.rs** — has solid A0/A2/A4/A9/A19 coverage for
  Guardian, Hexaghost, SlimeBoss, BronzeAutomaton, Collector, Champ,
  AwakenedOne, Donu, Deca, TimeEater, CorruptHeart. Gap:
  `collector_base_hp_and_spawn` (A0, hp=282) only asserts opening
  move; no post-roll MegaDebuff effect values at A0. Line 289-308 tests
  A2 MegaDebuff values but A0 values aren't pinned.
- **P2** Cultist, FungiBeast, Louse, Slaver, AcidSlime, SpikeSlime,
  Sentry, Looter, all Gremlins, Act 2 commons (Chosen, Mugger, Byrd,
  ShelledParasite, SnakePlant, Centurion, Mystic, Taskmaster,
  SphericGuardian, Snecko, BanditBear, BanditLeader, BanditPointy,
  BronzeOrb, TorchHead), Act 3 commons (Darkling, OrbWalker, Spiker,
  Repulsor, Exploder, WrithingMass, SpireGrowth, Maw, Transient,
  GiantHead, Nemesis, Reptomancer, SnakeDagger), Act 4 commons
  (SpireShield, SpireSpear) — **no ascension damage/HP scaling tests
  in any file**. Java scales commons at A2 (HP) and A17 (damage); none
  of that is locked down.

## Orphaned / disabled tests

- None. No `#[ignore]` attributes. No `|| true` tautologies remain —
  `test_enemies.rs:603-606` is a comment documenting that D158 already
  removed `heart_has_invincible` (which had exactly that bug) and
  pointing to the better-asserted replacement at
  `test_bosses.rs:509-525`.
- **P2** `test_enemies.rs:529-537 book_stab_count_increases` wraps its
  core check inside `if e.move_id == BOOK_STAB`. If the BIG_STAB path
  fires (Java num<15 branch), the `if` short-circuits and the test
  passes with zero assertions executed. Not strictly orphaned but
  effectively unreachable under `StsRandom::new(0)` which always yields
  num<15 → the BIG_STAB branch triggers first. Worth double-checking.
- **P2** `test_enemies.rs:216-224 rs_scrape_vuln` — entire assertion is
  inside `if e.move_id == RS_SCRAPE`. If the roll lands on something
  else (current seed happens to hit SCRAPE, but brittle), the assert is
  skipped silently.

## Self-seeding / fresh-RNG-per-call pattern

- **P1** **test_enemies.rs** uses `crate::seed::StsRandom::new(0)` as a
  fresh per-call RNG. `StsRandom::new(0).random_range(0..100)` yields
  a deterministic num — but always the same num, so every `roll_next_move`
  call in this file pulls the same number. The `num>=25` / `num>=40` /
  `num<33` branches in the probabilistic enemies (Nob, Lagavulin,
  Nemesis, Automaton, CorruptHeart — lines 485-497, 509-520, 547-558,
  566-578, 607-618) therefore only ever exercise one branch (whichever
  num=seeded-value lands in). This is effectively self-consistency
  masquerading as branch coverage. **test_enemy_ai.rs** fixes this with
  `roll_with_num(&mut e, N)` which takes explicit num, and
  **test_ai_rng_parity.rs** loops num=0..99 — those are correct.

## Duplicated coverage (specific)

- **P2** Guardian opening + mode-shift + offensive cycle:
  `test_enemies.rs:337-388` duplicates
  `test_bosses.rs:38-98,100-166`. Per-assert coverage is a subset of
  the boss file; keep the boss file canonical.
- **P2** Hexaghost activate + divider + 7-cycle:
  `test_enemies.rs:392-433` subset of `test_bosses.rs:100-198`.
- **P2** SlimeBoss sticky + full cycle + split thresholds:
  `test_enemies.rs:437-472` subset of `test_bosses.rs:200-230`.
- **P2** BronzeAutomaton spawn + hyper beam:
  `test_enemies.rs:562-578` (+ weak `>= 45` threshold) duplicates
  `test_bosses.rs:236-271` (exact `== 45`). Delete the weaker copy.
- **P2** CorruptHeart create + blood shots:
  `test_enemies.rs:598-618` (+ weak `>= 12` threshold) duplicates
  `test_bosses.rs:507-553` (exact `== 12`, `== 40`, full buff cycle).
- **P2** AwakenedOne slash + curiosity + phase:
  `test_enemies.rs:582-594` subset of `test_bosses.rs:365-402`.

## Coverage gaps

- **P0** `heart_debilitate` turn-0 debuff application (the very first
  intent) is covered as init state at `test_bosses.rs:512-515` but no
  test exercises the `apply_debuff_from_enemy` path through
  `combat_hooks::do_enemy_turns`. A regression where the Debilitate
  intent runs without landing WEAK/VULN/FRAIL would pass the init
  check and fail only in integration.
- **P1** Centurion + Mystic `BLOCK_ALL_ALLIES` / `HEAL_LOWEST_ALLY` /
  `STRENGTH_ALL_ALLIES` cross-enemy effects
  (`combat_hooks.rs:454-483`) are unit-tested nowhere in these four
  files. Their emission is asserted (`test_enemy_ai.rs:485,496,522`)
  but the allies-side post-execute effect is untested.
- **P1** Reptomancer `REPTO_SPAWN` minion spawn
  (`combat_hooks.rs:501-503` pushes `SnakeDagger` twice) — init state
  asserts Reptomancer begins with SPAWN, and `act3_patterns` at
  `test_enemy_ai.rs:815-816` re-rolls into SPAWN, but no test checks
  that after execute the enemy list grew by 2 SnakeDagger entries.
  Same gap for `COLL_SPAWN` (Collector pushes 2 `TorchHead`) and
  `BA_SPAWN_ORBS` (Automaton pushes 2 `BronzeOrb`).
- **P1** Lagavulin wake-up on damage (the non-sleep-turns-based wake):
  `lagavulin_wake_up` helper is called directly at
  `test_enemy_ai.rs:354`, not via the "took damage > 0" hook. Java
  wakes on damage OR sleep expiry; only expiry is tested end-to-end.
- **P1** Nemesis per-turn Intangible application: init asserts
  `INTANGIBLE == 0` (`test_enemies.rs:544`, correct per-Java), but no
  test verifies combat_hooks applies Intangible **at the start of each
  Nemesis turn**. Hook exists (verified at `combat_hooks.rs` Nemesis
  branch); no assertion wired.
- **P1** WrithingMass `REACTIVE` / `MALLEABLE` activation pathways
  (take-damage → reroll to BIG_HIT; player applies debuff → malleable
  stacks). `test_enemy_ai.rs:720` calls `writhing_mass_reactive_reroll`
  directly. No test drives it through `CombatEngine::deal_damage_to_enemy`
  → reactive chain. Same for MALLEABLE.
- **P1** CorruptHeart `BEAT_OF_DEATH` tick. Init is exact
  (`test_bosses.rs:517 == 1`, A19 `:528 == 2`), and combat_hooks
  emits the end-of-player-turn damage based on cards played; no test
  verifies the damage math.
- **P2** AcidSlime_M / AcidSlime_L corrosive-spit damage application
  (drops Slimed cards via `mfx::SLIMED`). Emission asserted at roll
  time; cards-added-to-discard is not checked.
- **P2** `BEAR_HUG` applies `DEX_DOWN` (init asserted at
  `test_enemy_ai.rs:411`); no post-execute test verifies player
  Dexterity actually drops.
- **P2** Snecko `CONFUSED` one-shot init (`test_enemy_ai.rs:408`). No
  post-execute test verifies `sid::CONFUSION` lands on player and
  persists for the rest of combat.
- **P2** Mugger / Looter escape + smoke bomb combo —
  `is_escaping = true` is asserted (`test_enemy_ai.rs:319,448`); the
  actual removal of the enemy from the combat state on escape is not
  tested.
- **P2** SphericGuardian initial 40 block decays — init block asserted
  at `test_enemy_ai.rs:405`; no test verifies the block carries or
  stacks correctly across turns through `combat_hooks`.

## Recommendation for PR #138

The Spiker fix closed the one active P0. The repo is in a supportable
state to merge. **Register-and-defer** the above as P1/P2 findings per
`feedback_audit_register_and_defer.md`. Main follow-up priorities:

1. Rewrite `test_enemies.rs` weak asserts to exact equality against
   Java values (lines 132, 157, 492, 507, 527, 535, 551, 573, 611).
2. Add ascension HP+damage+effect tests for at least one common
   per-act (Nob A2 StrengthDown, Snecko A17 Bite, Spiker A17 Thorns,
   SpireShield A19).
3. Exercise `combat_hooks::do_enemy_turns` end-to-end for at least
   Debilitate, REPTO_SPAWN, BEAT_OF_DEATH, and Nemesis Intangible.
4. Delete the duplicated boss coverage in `test_enemies.rs` lines
   337-388, 392-433, 437-472, 562-578, 582-594, 598-618 once the
   `test_bosses.rs` sibling is confirmed tighter (it is).
