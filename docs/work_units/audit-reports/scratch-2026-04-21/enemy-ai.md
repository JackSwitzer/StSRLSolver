# Enemy AI parity audit — Stage G follow-up (2026-04-21)

Read-only Java↔Rust re-audit of Act 1–4 enemy `getMove` bodies against the Stage D aiRng rewrite, triggered by the Spiker Thorns double-apply fix (commit `d16cdba2`). Goal: surface any remaining double-apply / mis-threshold / wrong-state-guard bugs.

## Summary

- **Total findings**: 2 new, 21 confirmed (re-verified against register D100–D159), 9 confirmed-clean.
- **New (not yet in register)**:
  - **P2** — BookOfStabbing A18+ stabCount increment on BigStab path missing
  - **P2** — Mugger turn-2 branch collapses Java's 50/50 Escape/BigSwipe into deterministic BigSwipe, plus A17+ escape-block +6 missing
- **Confirmed clean** (no deviation beyond what's already documented):
  - Spiker post-`d16cdba2` — single `mfx::THORNS` emission, combat_hooks applies once
  - Jaw Worm BELLOW/THRASH branches post-Stage-F swap
  - Collector MegaDebuff off-by-one post-Stage-F fix
  - Cultist first-turn Incantation + Ritual 3 chain (ignores num, matches Java)
  - Hexaghost 7-cycle orbActiveCount progression (num ignored)
  - Slime Boss firstTurn-only getMove + setMove chain (matches Java structure)
  - Guardian isOpen CHARGE/ROLL toggle with setMove chain
  - Awakened One Form 1 → Rebirth → Form 2 (lastMove/lastTwoMoves guards match)
  - Time Eater Haste-reset cadence (every 12th move, matches `moveCount == 12`)

## Confirmed clean (re-verified)

### Spiker — Thorns no longer double-applies
- **Rust**: `packages/engine-rs/src/enemies/act3.rs:71-88` — Buff branch sets intent + bumps `sid::COUNT` + queues `mfx::THORNS(2)`; does NOT bump `sid::THORNS` directly.
- **combat_hooks**: `packages/engine-rs/src/combat_hooks.rs:426-430` — `mfx::THORNS → add_status(sid::THORNS, amt)` (single write).
- **Verification**: Only one path writes `sid::THORNS`; Spiker emits +2/turn matching Java.

### Jaw Worm — BELLOW/THRASH dispatcher post-Stage-F
- **Rust**: `packages/engine-rs/src/enemies/act1.rs:11-52` — CHOMP for `num<25`, THRASH for `25<=num<55`, BELLOW for `num>=55` with guard fallbacks.
- **Java**: `decompiled/java-src/com/megacrit/cardcrawl/monsters/exordium/JawWorm.java:145-181`.
- **Verification**: Three branches and dominant-branch sub-roll fallbacks match Java (D129 closed).

### TheCollector — MegaDebuff `turns >= 3` post-Stage-F
- **Rust**: `packages/engine-rs/src/enemies/act2.rs:492`, comment at `:476-498` documents parity analogue to Java's post-increment `turnsTaken`.
- **Java**: `TheCollector.java:185` — `turnsTaken >= 3 && !ultUsed`.
- **Verification**: D130 closed, MegaDebuff queues for turn 4 as in Java.

## New findings

### BookOfStabbing: A18+ stabCount increment on BigStab path missing
- **Severity**: P2
- **Java**: `decompiled/java-src/com/megacrit/cardcrawl/monsters/city/BookOfStabbing.java:128-146` — `num<15 && !lastMove(BIG_STAB)` branch sets BigStab then does `if (ascensionLevel >= 18) ++stabCount`; same at `num>=15 && lastTwoMoves(STAB)` branch (L138-142).
- **Rust**: `packages/engine-rs/src/enemies/act2.rs:233-255` — BigStab branches at L246, L249 never bump `sid::STAB_COUNT`; only the Stab branches do.
- **Deviation**: At A18+, Java BigStab path also increments `stabCount`, so the subsequent Stab deals `(count+1)*6` instead of staying at the prior counter. Rust BigStab path never bumps, so the next Stab always emits at the same `stabCount` regardless of ascension.
- **Register**: new (related scope to D118 but specific to Book of Stabbing — not captured there)

### Mugger: turn-2 branch collapses Java's 50/50 Escape/BigSwipe + A17+ escape-block +6 missing
- **Severity**: P2
- **Java**: `decompiled/java-src/com/megacrit/cardcrawl/monsters/city/Mugger.java:86-114` — on turn 2 (`slashCount == 2` after takeTurn increment), Java does `aiRng.randomBoolean(0.5f) → setMove(byte 2, DEFEND)` (escape-block branch) ELSE `SetMoveAction(MOVES[0], 4, ATTACK, damage[1])` (BigSwipe). Also `getMove` itself only ever sets byte 1 (MUG) — the cycle is driven entirely by takeTurn `SetMoveAction` calls. Java escape-block branch at L133-136 applies `escapeDef+6` at A17+.
- **Rust**: `packages/engine-rs/src/enemies/act2.rs:44-59` — counts via `move_history.len()` and deterministically emits `MUGGER_BIG_SWIPE` at `turns == 2`, `MUGGER_SMOKE_BOMB (block 11)` at `last_move(BIG_SWIPE)`, then `MUGGER_ESCAPE` at `last_move(SMOKE_BOMB)`. No 50/50 split. Block amount is fixed 11 (no A17+ +6 bump).
- **Deviation**:
  1. Turn-2 is always BigSwipe (should be 50/50 with escape-block). Rust deals ~50% more damage on turn 2 than Java's expected value.
  2. No A17+ escape-block bonus (+6 block = 17 total instead of 11).
  3. Rust also doesn't model the SmokeBomb→Escape chain correctly: Java's escape-block *is* the escape setup (case 2 → sets byte 3 ESCAPE); Rust fabricates a separate SMOKE_BOMB move before escaping.
- **Register**: new. Similar structural class to D134/D135 (BanditLeader / BanditBear) — Java drives the cycle via `SetMoveAction` chains in `takeTurn`, Rust simulates via deterministic move-history inspection.

## Confirmed (already in register)

Re-verified the following Stage G register entries remain accurate against current source:

| ID | Enemy | Status |
|---|---|---|
| D100 | TheCollector REVIVE (case 5) completely missing | confirmed — `roll_collector` at `act2.rs:471-509` has no Revive branch; explicit "deferred: needs minion-dead signal" comment at L481 |
| D101 | CorruptHeart A9-vs-A19 HP-gate conflation | confirmed — `mod.rs:785-803` still gates on `hp >= 800` |
| D133 | BronzeAutomaton HyperBeam fires turn 5 instead of turn 6 | confirmed — `act2.rs:343-344` uses `move_history.len() >= 4` which includes SPAWN turn |
| D134/D135 | BanditLeader / BanditBear re-emit opener every 3rd turn | confirmed — `act2.rs:307-329` fallthrough cycles re-enter MOCK/BEAR_HUG every 3rd turn |
| D136 | GremlinLeader ignores aliveCount | confirmed — `act2.rs:257-268` is a pure 3-state cycle, no `numAliveGremlins` branch |
| D137 | Champ Phase-2 Anger threshold off-by-one | confirmed — `act2.rs:389` still `hp <= max_hp / 2` |
| D138 | Champ Defensive Stance missing `forgeTimes < 2` cap | confirmed — `act2.rs:438-447` bumps `sid::FORGE_TIMES` but never gates on it |
| D139 | Snecko collapses two-threshold branches to `num<50` | confirmed — `act2.rs:290-305` lacks `num<40 → TAIL` slice |
| D140 | WrithingMass ReactivePower never wired | confirmed — `writhing_mass_reactive_reroll` defined in `act3.rs` but not referenced from `combat_hooks.rs` |
| D141 | Transient ATTACK damage direction reversed | confirmed — `act3.rs:272-281` |
| D142 | Exploder skips UNKNOWN turn 2 | confirmed — `act3.rs:100-114` |
| D143 | CorruptHeart slot 0 deterministic Blood Shots | confirmed — `act4.rs:124-128` always picks BLOOD_SHOTS |
| D144 | CorruptHeart A4-A8 scaling gated on HP threshold | confirmed — `mod.rs:785-803` |
| D146 | FungiBeast ignores `num` | confirmed — `act1.rs:58-67`, signature `_num` |
| D147/D148 | SpikeSlime_M/L ignore `num` | confirmed — `act1.rs:217-241`, signatures `_num` |
| D149 | AcidSlime_S threshold `num<40` vs Java `num<30` + fabricated guard | confirmed — `act1.rs:155-171` |
| D150 | Louse anti-repeat single-move vs Java `lastTwoMoves` | confirmed — `act1.rs:69-107` |
| D151 | SlaverBlue extra `!last_move(STAB)` guard | confirmed — `act1.rs:115-119` |
| D152 | GremlinWizard 2-turn cycle vs Java 3-turn | confirmed — `act1.rs:307-315`; Java needs `currentCharge == 3` gating |
| D153 | GremlinTsundere dispatcher arm empty | confirmed — `mod.rs:853` empty match arm |
| D154 | Lagavulin 1:1 vs Java 2:1 attack:debuff ratio | confirmed — `act1.rs:341-362`; no `debuffTurnCount` status |
| D155/D156 | Sentry BOLT/BEAM swap + first-move positional | confirmed — `act1.rs:371-378`; `mod.rs` create_enemy seeds BOLT regardless of index |
| D157 | Looter AI RNG stream off-by-one | confirmed — `act1.rs:243-267` consumes num on turn 1 unlike Java |
| D118 | Ascension scaling absent across enemies | confirmed — WrithingMass A0 damage values at `act3.rs:130-136`; Taskmaster woundCount fixed at 1 at `act2.rs:273`; GremlinFat A17+ Frail absent at `mod.rs:453-457` (all subsumed by D118 scope) |

## Double-apply check (post-Spiker)

Audited every `roll_*` for the Spiker bug pattern (status bumped in roll_* AND mfx queued for combat_hooks). No additional cases found:

- **SPIKER THORNS** (fixed by `d16cdba2`) — single emission path
- **STRENGTH / STRENGTH_BONUS / ARTIFACT / BEAT_OF_DEATH / PAINFUL_STABS** (CorruptHeart Buff branch, `act4.rs:137-149`) — checked; emits only via `add_effect(mfx::*)`, no direct `sid::*` bump
- **RITUAL** (Cultist, `mod.rs:407-410`) — first-turn-only, mfx-queued; combat_hooks doesn't double-apply
- **STRENGTH / DEX_DOWN / VULNERABLE / FRAIL / WEAK / BURN** — all routed through `add_effect`; no `roll_*` writes `sid::*` directly for these

No further Thorns-class double-apply bugs found.

## Notes on scope

- **Did not re-audit**: D97 (GremlinTsundere also covered by D153), D98 (ApologySlime default-case fallthrough), D99 (SphericGuardian Barricade+Artifact passives), D102 (SnakeDagger explosion damage type), D132 (Byrd grounded synthesis — already tracked for follow-up).
- **Confidence**: High for all "confirmed clean" entries; the Spiker fix is structurally correct and leaves no other mfx/sid double-writes.
- **Recommended next step**: promote the 2 new findings (BookOfStabbing A18+ / Mugger turn-2) into `parity-deviations-register.md` as D160/D161 alongside the existing Stage G follow-ups.
