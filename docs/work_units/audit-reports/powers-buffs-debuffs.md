# Parity audit: Powers / buffs / debuffs (universal + enemy-generic)

**Date:** 2026-04-21
**Auditor:** Opus 4.7 (subagent)
**Scope:** Java `powers/` (excluding Watcher-specific) vs Rust `powers/{buffs,debuffs,enemy_powers}.rs` + `defs/` + `status_ids.rs`
**Powers reviewed:** ~40 Java `*Power.java` sources
**Deviations found:** 50 (P1-P50)

## Summary

Audit covered all player buffs, player debuffs, and generic (non-enemy-specific) enemy powers, excluding Watcher-specific powers per scope. Three high-impact findings:

1. **P1 MalleablePower never resets** — `deal_damage_to_enemy` increments +1 on every hit; Java `atEndOfTurn` resets to `basePower`. Multi-turn fights vs Chosen/Stone Wyrm inflate block-per-hit unboundedly.
2. **P2 Start-of-turn vs StartOfTurnPostDraw conflated** — DemonForm/Brutality/NoxiousFumes/ToolsOfTheTrade run pre-draw in Rust; Java fires post-draw. Brutality+Draw sequence inverted, NoxiousFumes Artifact off-by-one, DemonForm STR available one turn earlier.
3. **P3 SadisticPower missing exclusions** — `apply_debuff_with_sadistic` doesn't filter (source==owner, target!=owner, PowerType.DEBUFF only, Shackled exclusion). Double-dips Shackled + self-applied debuffs.

## Missing / unwired powers

| Power | Rust status | Java ref | Impact |
|---|---|---|---|
| PhantasmalPower | No sid, no def | `PhantasmalPower.java` | Potion (Ghost in a Jar) — 3-turn DoubleDamage. Moderate. |
| ReboundPower | sid? def? | `ReboundPower.java` | Silent — replay next non-power. Silent parity gap. |
| BiasPower | No sid, no def | `BiasPower.java` | Defect — -N Focus/turn. Defect damage gap. |
| BackAttackPower | No def | `BackAttackPower.java` | Awakened One backstab. |
| StasisPower | No def | `StasisPower.java` | Defect Stasis death trigger. |
| RechargingCorePower | No sid, no def | `RechargingCorePower.java` | Defect Recharge +1E/3t. |
| TimeMazePower | No def | `TimeMazePower.java` | Nightmare potion. |
| AmplifyPower | No def | `AmplifyPower.java` | Silent — replay next power. |
| LoopPower | No def | `LoopPower.java` | Defect first-orb double evoke. |
| HelloPower | No def | `HelloPower.java` | Relic — random common card/turn. |
| MayhemPower | No def | `MayhemPower.java` | Relic — play top card/turn. |
| PanachePower | No def | `PanachePower.java` | Relic — every 5 cards DamageAll. |
| HeatsinkPower | No def | `HeatsinkPower.java` | Defect — draw on power play. |
| StormPower | No def | `StormPower.java` | Defect — channel Lightning on power play. |
| ElectroPower | `sid::ELECTRO` exists | `ElectroPower.java` | Watcher Electrodynamics — wiring unverified. |
| ReactivePower | `sid::REACTIVE` exists | `ReactivePower.java` | Writhing Mass RollMove on attacked — unverified. |
| ShiftingPower | No def | `ShiftingPower.java` | Byrds/Darkling STR transfer on attack. |

## Deviations

### P1 — MalleablePower: no atEndOfTurn reset [bug]
- **Rust:** `packages/engine-rs/src/engine.rs:2521-2527` — `MALLEABLE` increments +1 on every hit; no reset.
- **Java:** `MalleablePower.atEndOfTurn(isPlayer)` sets `amount = basePower`.
- **Impact:** Multi-turn fights accumulate indefinite block-per-hit gain.
- **Fix:** reset `MALLEABLE` to base (new `MALLEABLE_BASE` sid) on enemy end-of-turn.

### P2 — StartOfTurn vs StartOfTurnPostDraw timing [bug]
- **Rust:** `packages/engine-rs/src/powers/buffs.rs::process_start_of_turn` applies all start-of-turn effects pre-draw.
- **Java atStartOfTurnPostDraw (post-draw):** DemonForm, Brutality, NoxiousFumes, ToolsOfTheTrade
- **Java atStartOfTurn (pre-draw):** Berserk, InfiniteBlades, NextTurnBlock, DrawCardNextTurn, Phantasmal, Bias, RechargingCore, Loop, Mayhem, Hello, Panache
- **Impact:** Brutality draw-then-damage order wrong; NoxiousFumes Artifact absorption off-by-one; DemonForm STR available one turn earlier than Java; ToolsOfTheTrade discard target set differs.
- **Fix:** split into pre-draw/post-draw passes or add `process_start_of_turn_post_draw` hook.

### P3 — SadisticPower missing exclusions [bug]
- **Rust:** `powers/debuffs.rs:215-227` `apply_debuff_with_sadistic` fires on any applied debuff.
- **Java:** `SadisticPower.onApplyPower` filters: `source != owner` returns; `type != DEBUFF` returns; `ID == "Shackled"` returns; `target == owner` returns.
- **Impact:** Fires on Shackled (double-dip with GainStrength recovery); fires on self-applied LoseStrength/LoseDexterity.
- **Fix:** caller must check debuff type + source/target identity + Shackled exclusion before Sadistic payload.

### P4 — ThornsPower / FlameBarrier HP_LOSS bypass [unverified]
- **Java:** `ThornsPower.onAttacked` requires `info.type != HP_LOSS && info.type != THORNS && info.owner != this.owner`.
- **Action:** verify Rust attack resolver passes `damage_type` and filters HP_LOSS + self-attribution.

### P5 — RetainCardPower vs RunicPyramid vs Equilibrium gating [unverified]
- **Java:** end-of-turn hand discard skipped when ANY of {RunicPyramid, Equilibrium, RetainCardPower} present.
- **Action:** verify Rust checks all three.

### P6 — AccuracyPower stack shiv update [deferred]
- **Java:** `AccuracyPower.stackPower` mutates all existing Shivs' damage +amount in place.
- **Action:** Rust likely computes at play-time (architecturally cleaner); verify no stale-cache UI side-effect.

### P7 — BerserkPower energy gating [unverified]
- **Java:** `GainEnergyAction(amount)` uncapped, not bound by `max_energy`.
- **Action:** verify Rust doesn't cap.

### P8 — GainStrengthPower (Shackled recovery) timing [unverified]
- **Java:** `atEndOfTurn` restores STR equal to amount, fires at owner end-of-turn.
- **Action:** verify Rust fires at Shackled-target end-of-turn, not round end.

### P9 — ConstrictedPower end-of-turn damage [unverified]
- **Java:** `atEndOfTurn(isPlayer)` deals THORNS damage (ignores block).
- **Action:** verify Rust wires the end-of-turn damage tick; THORNS type (block-ignore).

### P10 — EntanglePower cleanup [unverified]
- **Java:** `atEndOfTurn(isPlayer)` fully removes when `isPlayer`, doesn't decrement.
- **Action:** confirm Rust full-removes at player end-of-turn.

### P11 — BufferPower: stack on damage reduction (augments D48) [unverified]
- **Java:** only reduces Buffer when `damageAmount > 0` post-block AND `type != HP_LOSS`.
- **Action:** verify `check_buffer` call site filters zero-damage and HP_LOSS.

### P12 — PoisonPower cap at 9999 [intentional]
- Rust no cap. Extreme but affects recorded-Java replay determinism.

### P13 — Strength/Dex/Focus ±999 clamp [intentional]
- Rust no clamp. Affects infinite loops (Berserk + LimitBreak).

### P14 — DarkEmbrace: areMonstersBasicallyDead check [unverified]
- **Java:** skips draw when combat effectively over.
- **Action:** verify Rust gates Dark Embrace draw on living-enemy-count > 0.

### P15 — FireBreathingPower CURSE inclusion [unverified]
- **Java:** fires on STATUS OR CURSE, uses THORNS, damages all enemies.
- **Action:** verify Rust fires on both types, THORNS block-ignore, all-enemies AoE.

### P16 — EvolvePower NoDraw interaction [unverified]
- **Java:** skips Evolve draw when owner has NoDraw.
- **Action:** verify Rust respects `sid::NO_DRAW`.

### P17 — EnvenomPower NORMAL + target!=owner [unverified]
- **Java:** applies Poison only on NORMAL damage with `damageAmount > 0` and `target != owner`.
- **Action:** verify Rust filters damage type, non-zero post-block, non-self target.

### P18 — FlameBarrier remove timing [unverified]
- **Rust:** `remove_flame_barrier` called in `process_start_of_turn`.
- **Java:** `atEndOfTurn(isPlayer)` — removed at end of PLAYER's turn (before enemy turn). Current Rust keeps Flame Barrier active during enemy turn = FREE RETALIATION.
- **Severity:** high for Ironclad defense.
- **Action:** verify timing of `RemoveSpecificPowerAction` relative to enemy intents.

### P19 — JuggernautPower RNG stream [unverified]
- **Java:** `getRandomMonster(null, true)` uses `cardRandomRng`, not `monsterRng`.
- **Action:** verify Rust uses card RNG (not AI RNG), fires on EVERY GainBlockAction, uses THORNS.

### P20 — AfterImagePower: ANY card triggers [unverified]
- **Java:** no card-type filter.
- **Action:** confirm Rust fires on Skill/Attack/Power/Status/Curse alike.

### P21 — FeelNoPainPower: per-exhaust granularity [unverified]
- **Java:** fires once per exhausted card.
- **Action:** verify Rust onExhaust dispatch is per-card, not per-batch.

### P22 — ForcefieldPower NORMAL-only nullify [unverified / Watcher-borderline]
- **Java:** nullifies NORMAL only; THORNS, HP_LOSS pass through.
- **Action:** verify filter.

### P23 — LockOnPower Lightning +50% [unverified]
- **Java:** +50% NORMAL incoming + Lightning orb +50% check.
- **Action:** verify both paths in Rust.

### P24 — Heatsink/Storm POWER-play triggers [deferred]
- Defect-only. Missing.

### P25 — DevaFormPower stack escalation [unverified]
- **Java:** `atStartOfTurn` grants energy + self-increments amount +1.
- **Impact:** if stack doesn't grow, Deva Form stuck at +1E forever.
- **Action:** verify `apply_deva_form` both grants AND increments stack.

### P26 — Ginger/Turnip Artifact ordering [verified clean]
- Artifact first, Ginger/Turnip next — matches Java.

### P27 — SlowPower atEndOfRound reset [unverified]
- **Java:** `atEndOfRound` sets `amount = 0` (direct reset).
- **Action:** verify `reset_slow` direct-sets to 0 and fires at end-of-round.

### P28 — TimeWarpPower 12-play trigger [unverified]
- **Java:** at 12 plays: apply +2 STR, force end-turn, reset counter.
- **Action:** verify all three legs.

### P29 — FadingPower suicide at 1 [unverified]
- **Java:** suicide action at amount==1.
- **Action:** verify death timing (end-of-turn vs start-of-turn).

### P30 — ExplosivePower suicide + 30 THORNS [unverified]
- **Java:** at countdown 1: suicide + 30 THORNS damage to player.
- **Action:** if enemy exists in Rust, verify THORNS block-piercing.

### P31 — BlurPower atEndOfRound decrement [unverified]
- **Java:** `atEndOfRound` not `atEndOfTurn`.
- **Action:** verify.

### P32 — FlightPower per-hit decrement [unverified]
- **Java:** decrement only on NORMAL + damage>0; 0.5x in `atDamageReceive` (not Final).
- **Action:** verify filter and multiplier placement.

### P33-P40 — Deferred (relic/class-scope)
Panache, Mayhem, Hello, Storm, Heatsink, Loop, BackAttack, Amplify, Phantasmal, Bias. All `deferred`.

### P41 — ReactivePower (Writhing Mass) [unverified]
- **Java:** `onAttacked` if NORMAL + dmg>0 + !dying → RollMoveAction.
- **Action:** verify Rust has RollMove dispatch for WM.

### P42 — RitualPower skipFirst [augments D54]
- **Java:** ctor sets `skipFirst = !isPlayer`; first end-of-round skipped.
- **Action:** verify D54 fix implements skipFirst flag.

### P43 — GrowthPower skipFirst [unverified]
- Same pattern as Ritual.

### P44 — SporeCloudPower onDeath [cross-ref D68/D82]

### P45 — BeatOfDeathPower per-card THORNS [unverified]
- **Java:** THORNS on every card play (Awakened One F3 P2).
- **Action:** verify per-card (not per-attack) + THORNS.

### P46 — ModeShiftPower damage accumulation [unverified]
- **Java:** accumulates damage dealt (post-block), not raw incoming.
- **Action:** verify Rust post-block vs raw.

### P47 — DrawReduction end-of-player-turn cleanup [unverified]

### P48 — DoubleDamagePower damage-order placement [unverified]
- **Java:** `(base + Str) * DoubleDamage * Weak * Stance * Vuln` — specific multiplicative order.
- **Action:** verify Rust damage calc order.

### P49 — Equilibrium hand-retention [unverified]

### P50 — TheBombPower THORNS detonation [unverified]
- **Java:** detonates at countdown 0, THORNS, block-piercing.
- **Action:** verify THORNS type.

## Items verified clean

- **Slow damage multiplier** (`powers/debuffs.rs:slow_damage_multiplier`): `1.0 + 0.1 * n` for NORMAL.
- **Intangible cap** (`powers/debuffs.rs:modify_damage_receive`): post-reduction cap at 1.
- **Invincible tracked** — separate per-turn tracker leaves base intact.
- **Poison tick** (`tick_poison`): decrements by 1, damages by stack.
- **LoseStrength / LoseDexterity / WraithForm Dex** (`apply_*`): all match Java patterns.
- **Ginger/Turnip Artifact ordering** — matches Java.
- **D59 Weak/Vuln/Frail justApplied** — `*_JUST_APPLIED` parallel flags verified.

## Follow-up questions

1. **P1 Malleable reset:** Track `MALLEABLE_BASE` status, or hardcode `3`?
2. **P2 StartOfTurn split:** Should be new phase between draw and player-actionable state?
3. **P3 Sadistic source tracking:** Does any existing code track debuff source identity?
4. **P18 FlameBarrier:** Repro — play FlameBarrier, end turn, confirm next enemy attack does NOT reflect thorns.
5. **P25 Deva Form:** Does `apply_deva_form` self-increment the stack?
