# Parity audit: Damage pipeline and combat turn flow

**Date:** 2026-04-21
**Auditor:** Opus 4.7 (subagent)
**Scope:** Java `AbstractCreature.damage()` / `DamageAction` / `AbstractMonster.takeTurn()` / `AbstractPlayer.applyStartOfTurn*` / `applyEndOfTurn*` vs Rust `packages/engine-rs/src/{damage.rs, combat_hooks.rs, engine.rs}` (specifically `end_turn`, `start_player_turn`, `do_enemy_turns`, `deal_damage_to_*`, `player_lose_hp`, `record_enemy_hp_damage`, `tick_poison`, `calculate_incoming_damage`).
**Files reviewed:** `engine.rs` (3900 LOC), `combat_hooks.rs` (~400 LOC), `damage.rs` (~440 LOC), `powers/debuffs.rs` (~300 LOC), `effects/interpreter.rs`, `effects/runtime.rs`, `card_effects.rs`, `run.rs`; Java oracles for `AbstractCreature`, `AbstractMonster`, `AbstractPlayer`, `DamageAction`, `HPLossAction`, `DamageInfo`, `FlightPower`, `ConstrictedPower`, `ThornsPower`, `FlameBarrierPower`, `StaticDischargePower`, `PenNibRelic`, `TungstenRod`, `Torii`, `CurlUpPower`, `SpikerPower`.
**Deviations found:** 14 (DM1–DM14)

## Summary

This is the most-used hot path in the engine (every enemy intent, every attack, every poison tick flows through it) and the highest-leverage source of silent parity drift. The overall pipeline is largely correct for the common case (player attacks enemy, enemy attacks player with block), but several systemic gaps compound over long runs:

1. **DM1 — `Target::Player` effect damage bypasses the entire Java damage pipeline.** `engine.deal_damage_to_player` runs only `block → player_lose_hp`. No Wrath 2x, no Vulnerable 1.5x, no Intangible cap, no Torii/Tungsten, no Plated Armor decrement, no Static Discharge. Several effect hooks (Reaper self-damage, Hemokinesis, Reckless Charge, Feed, potion self-hits, monster `DamageEffect { target: Player }`) route through this shortcut. Any power or relic content that targets `Player` via the effects DSL silently skips powers. This is currently the single largest under-the-radar bug in the damage path.
2. **DM2 — Flight decrements unconditionally, before block / before confirming a hit landed.** `deal_damage_to_enemy` decrements `FLIGHT` whenever `flight > 0`, even if Slow-adjusted damage is 0, even if Invincible caps to 0, even if the attack is fully blocked, and even on the lethal blow. Java `FlightPower.onAttackedToChangeDamage` is `info.type == NORMAL && damageAmount > 0`, and Java only decrements on `wasHPLost`. Every multi-hit attack vs Byrds now removes extra Flight stacks per turn than Java would.
3. **DM3 — Constricted / player-poison / player self-damage feed `player_lose_hp` directly, skipping `DamageInfo.applyPowers`.** Rust gates HP_LOSS through `damage::apply_hp_loss` (Intangible + Tungsten Rod only). Java's `HPLossAction` additionally fires `AbstractCreature.wasHPLost` power hooks (for Ironclad `IntangiblePlayer`, Rupture is separate from damage), `onInflictDamage` on the source for Constricted, and the `onLoseHp` relic callback chain. Our Rupture-on-HP-loss in `player_lose_hp` is correct; the missing pieces are source-side hooks (mainly `onInflictDamage`).
4. **DM4 — Retaliation (Thorns, FlameBarrier, Static Discharge, SharpHide on player, Vajra / Anchor on enemies of player) uses raw `entity.hp -= x` arithmetic, not the damage pipeline.** Block, Intangible, and Invincible on the retaliator's target are all bypassed. FlameBarrier vs Intangible enemies deals full damage in Rust, 1 in Java. (Overlaps existing parity-deviations-register entry D58.)
5. **DM5 — `tick_poison` (player) decrements poison simultaneously with HP loss inside `end_turn` instead of via `HPLossAction`.** Enemy poison tick goes through `record_enemy_hp_damage` which fires death hooks correctly; player poison tick bypasses the source-power hook chain entirely (no `onInflictDamage`, no `wasHPLost` on player's poison-apply source). `JUST_APPLIED` flag for Weak/Vulnerable/Frail is applied unconditionally when `apply_debuff_from_enemy` succeeds; Java gates it on `AbstractPower.justApplied = (source != null && !source.isPlayer && turnHasEnded)`. Pre-combat buff apps by enemy powers (e.g., start-of-combat Weak) will skip the first decrement in Rust when Java would not.

These five findings account for the bulk of silent damage drift. Everything else (DM6–DM14) is narrower but documented below.

## Damage pipeline diff (player → enemy, enemy → player)

| Step | Java order | Rust location | Match? |
|---|---|---|---|
| Card base damage + Str + Vigor | `AbstractCard.calculateCardDamage` | `damage::calculate_damage_full` | OK |
| Pen Nib 2x | `applyPowers` on attacker | `hooks_complex.rs` feeds `pen_nib` into `calculate_damage_full` | OK |
| Double Damage (HandOfGreed/etc.) | `applyPowers` on attacker | `calculate_damage_full(double_damage)` | OK for hook call, unverified for card effects |
| Weak 0.75 (Paper Crane 0.60) | `applyPowers` on attacker | `calculate_damage_full(weak, weak_paper_crane)` | OK |
| Stance mult (Wrath 2x / Divinity 3x) | `calculateDamageGivenToTarget` | `calculate_damage_full(stance_mult)` | OK |
| Vulnerable 1.5 (Paper Frog 1.75, Odd Mushroom 1.25 on player) | `calculateDamageReceive` on target | `calculate_damage_full(vulnerable, vuln_paper_frog)`, `calculate_incoming_damage(odd_mushroom)` | OK |
| Flight 0.5 on enemy | `calculateDamageReceive` on target | `calculate_damage_full(flight)` for card path AND `deal_damage_to_enemy` separate multiplier | **Partial — double application risk**. `deal_damage_to_enemy` multiplies again (line 2480) after interpreter already halved via `calculate_damage_full`. See DM2. |
| Intangible cap (max 1 post-multiplier) | `calculateDamageReceive` | Both `calculate_damage_full(intangible)` and `calculate_incoming_damage(intangible)` | OK |
| Slow +10%/stack on enemy | `SlowPower.atDamageReceive` | `deal_damage_to_enemy` via `slow_damage_multiplier` | OK |
| Invincible cap (Heart/Donu/Deca) | `InvinciblePower.atDamageReceive` | `apply_invincible_cap_tracked` inside `deal_damage_to_enemy` | OK |
| Floor to int | `(int) damage` | `(damage as i32).max(0)` | OK |
| Block absorb | `decrementBlock` | `deal_damage_to_enemy` / `deal_damage_to_player` / `calculate_incoming_damage` | OK |
| Torii (unblocked 2–5 → 1) | `Torii.onLoseHp` | `calculate_incoming_damage(torii)` | OK for enemy-intent path; **missing** for `deal_damage_to_player` path (DM1) |
| Tungsten Rod (-1 HP) | `TungstenRod.onLoseHp` | `calculate_incoming_damage(tungsten_rod)` + `apply_hp_loss(tungsten_rod)` | OK for intent path; **missing** for `deal_damage_to_player` path (DM1) |
| `wasHPLost` power hook | `AbstractCreature.damage` | No analogue on player; `player_lose_hp` emits `OnPlayerHpLoss` trigger (relics only) | **Gap** (DM3) |
| `onInflictDamage` on source | `AbstractCreature.damage` caller | None | **Gap** (DM3/DM5) |
| `onAttacked` on target | `CurlUpPower.onAttacked`, `MalleablePower.onAttacked` | `deal_damage_to_enemy` fires Curl Up and Malleable inline | OK |
| Plated Armor decrement | `PlatedArmor.onLoseHp (NORMAL)` | `combat_hooks.rs` enemy-intent path only | OK for intent; **missing** for `deal_damage_to_player` path (DM1) |
| Static Discharge channel on unblocked | `StaticDischargePower.onLoseHp (NORMAL)` | `combat_hooks.rs` enemy-intent path only | OK for intent; **missing** for `deal_damage_to_player` path (DM1) |
| Rupture STR on HP loss | `Rupture.onLoseHp (NORMAL or HP_LOSS)` | `player_lose_hp` | OK |
| HP\_LOSS variant (Hemokinesis/Reaper/Reckless/potion self) | `HPLossAction` → `player.damage(DamageInfo(null, n, HP_LOSS))` → skips block but still calls powers | `player_lose_hp(n)` direct | **Partial — Intangible/Tungsten missing at source**. Most callsites precompute via `apply_hp_loss` (OK), but some (`effects/interpreter.rs:1039/1044`, `card_effects.rs:229`, `hooks_simple.rs:112/235/240`) call `player_lose_hp` with raw base numbers. See DM3. |

## Turn flow diff

| Phase | Java order (`AbstractPlayer.applyStartOfTurn*` / `AbstractMonster.takeTurn` / `AbstractRoom.endTurn`) | Rust order (`engine.rs` `end_turn` / `start_player_turn` + `combat_hooks::do_enemy_turns`) | Match? |
|---|---|---|---|
| End-turn start | `applyEndOfTurnTriggers` + `atEndOfTurnPreEndTurnCards` | `emit_event(TurnEnd)` | OK |
| Discard on end-turn | `AbstractPlayer.applyEndOfTurnTriggers` → queues `DiscardAtEndOfTurnAction` | `engine.rs` explicit discard after TurnEnd | OK |
| Constricted tick | `ConstrictedPower.atEndOfTurn` → `DamageAction(HP_LOSS)` | `engine.rs:1509` direct `apply_hp_loss` + `player_lose_hp` | Partial (DM4) |
| Regen on player | `RegenerationPower.atEndOfTurn` → heal + decrement | `engine.rs:1522` heal + decrement | OK (matches D61/D74 fix) |
| Player poison tick | `PoisonPower.atEndOfTurn` on player target | `engine.rs:1529` inline | Partial (DM5) |
| Enemy turn execution | `AbstractMonster.takeTurn` in order | `do_enemy_turns` iterates `state.enemies` in order | OK |
| Enemy intent-attack | `attack()` → `DamageInfo(NORMAL)` → `player.damage()` → `wasHPLost` chain | `execute_enemy_move` → `calculate_incoming_damage` → `player_lose_hp` | Partial (DM3) |
| Enemy Intangible / debuff decrement | After all `monsters.takeTurn` via `applyEndOfTurnPowers` | `engine.rs:1554` gated behind `!skip_enemy_turn` | OK (D50 fix applied) |
| Enemy JUST\_APPLIED clear | `AbstractPower.atEndOfRound` (justApplied → skip first) | `decrement_debuffs` reads flag, clears, skips | OK for normal case, wrong gate (DM14) |
| Start player turn | `AbstractPlayer.applyStartOfTurn` | `start_player_turn` emits `TurnStart`, decays block, draws | Mostly OK (see D39/D40) |
| Draw 5 | `AbstractRoom.skipMonsterTurn` + `StartOfTurnCardsAction` | `engine.rs` draw after block decay | OK |
| `StartOfTurnPostDraw` powers | `AbstractPlayer.applyStartOfTurnPostDrawPowers` | `emit_event(TurnStartPostDraw)` | OK (P2 fix applied per powers-buffs-debuffs report) |
| Enemy turn skipped (Vault) | `skipMonsterTurn = true` → `monsters.applyEndOfTurnPowers` still fires in Java (gated in Rust) | Gated inside `!skip_enemy_turn` branch | Matches D50 — Java actually gates the entire end-of-round power pass, so Rust is correct |

## Deviations

### DM1 — `deal_damage_to_player` bypasses the entire Java damage pipeline (HIGH)
- **Symptom:** When the effects DSL deals damage to `Target::Player` (`effects/runtime.rs:1715`), the value is treated as already-final and routed through `player.block.min(damage)` → `player_lose_hp`. No Wrath 2x, no Vulnerable 1.5x, no Intangible cap, no Torii, no Tungsten Rod, no Plated Armor decrement, no Static Discharge channel.
- **Java:** `AbstractCreature.damage(DamageInfo info)` always runs `info.applyPowers` (which invokes every `atDamageReceive` power) and then `calculateDamageReceive` (Vulnerable, Intangible, Flight), and `decrementBlock`, then power `wasHPLost`.
- **Impact:** Any custom-damage effect targeting Player silently misses relic/power modifiers. Includes: monster `DamageEffect` variants, some card self-damage effects, potential future content. Player currently takes flat unmodified damage through this route.
- **Fix:** Route `deal_damage_to_player` through `damage::calculate_incoming_damage` with full context (Wrath, Vulnerable, Intangible, Torii, Tungsten, Odd Mushroom) and the same post-block Plated Armor / Static Discharge chain as `combat_hooks::on_enemy_attack_player`. Consider extracting a single `apply_incoming_damage(engine, raw, source_kind)` helper used by both enemy-intent and effect-DSL paths.

### DM2 — Flight decrements unconditionally, bypassing Java's `damage > 0 && NORMAL hit` gate (HIGH)
- **Symptom:** `engine.rs:2478–2485` decrements `FLIGHT` by 1 every time `deal_damage_to_enemy` is called with `flight > 0`, even when Slow-multiplied damage is 0, or when the attack is fully blocked, or on the lethal blow. The decrement happens before block, and before the Invincible cap.
- **Java:** `FlightPower.onAttackedToChangeDamage(info, damageAmount)` decrements by 1 if `info.type == NORMAL && damageAmount > 0`. Post-block HP loss is irrelevant; pre-block raw-damage > 0 is the only condition.
- **Compounding issue:** The effects DSL card path (`effects/hooks_complex.rs:580–593`) already multiplies by 0.5 via `calculate_damage_full(flight)` BEFORE calling `deal_damage_to_enemy`, which then halves again at line 2482. If the attack goes through `calculate_damage_full` first, Flight is applied twice. Need to trace which paths feed pre-multiplied vs raw damage.
- **Impact:** Multi-hit attacks vs Byrds (start with Flight 3) incorrectly decrement Flight on 0-damage and blocked hits. Byrds tier list shifts in Rust.
- **Fix:** Gate decrement on `damage_after_slow > 0` and skip when Invincible cap produces 0. Audit whether `deal_damage_to_enemy` should be the sole Flight site and card paths should pass `flight=false` into `calculate_damage_full`.

### DM3 — Player-side `onLoseHp` / `wasHPLost` chain incomplete (HIGH)
- **Symptom:** `player_lose_hp` only fires `OnPlayerHpLoss` event (relic-only) and Rupture. Does not fire any `AbstractPower.wasHPLost` analogue or source-side `onInflictDamage`.
- **Java:** `AbstractCreature.damage()` calls every power's `wasHPLost(info, damageAmount)` in order, then `for each relic: onLoseHp(damage)`. Source of the damage calls `onInflictDamage` after.
- **Impact:** Any player power with `wasHPLost` logic silently no-ops. Known Watcher / Ironclad powers rely on this (e.g., `IntangiblePower` handles its self-expiry, `BerserkPower` is STR-not-HP so OK). Source-side `onInflictDamage` misses enemy power triggers (e.g., Gremlin Leader patrol mechanics are unverified).
- **Fix:** Emit a `Trigger::OnPlayerWasHPLost` event inside `player_lose_hp` after the HP write and before the revive check. Route all player-power `wasHPLost` logic into handlers for this trigger. For source-side `onInflictDamage`, add an emit after `record_enemy_hp_damage` for enemy-targeted damage (event exists as `DamageResolved` but not currently fanned out to source powers).

### DM4 — Retaliation (Thorns / FlameBarrier / Static Discharge / SharpHide) uses raw arithmetic, skipping pipeline (HIGH, overlaps D58)
- **Symptom:** Confirmed via `parity-deviations-register.md` D58. Thorns on enemy hit by player attack, FlameBarrier on enemy hit, Static Discharge on player hit, and SharpHide on player attack all subtract HP directly or route via `player_lose_hp` without `DamageInfo(THORNS).applyPowers`.
- **Java:** Thorns power queues `DamageAction(owner, DamageInfo(owner, thorns, THORNS))`. THORNS damage is affected by target Intangible (cap 1), target block, Invincible, and target `wasHPLost`.
- **Impact:** FlameBarrier vs Intangible enemy deals full stack in Rust, 1 in Java. Thorns damage to Intangible player should cap 1. Static Discharge Lightning is routed through direct `entity.hp -=` at `combat_hooks.rs:205–213`, skipping Invincible / Curl Up / death hooks (`record_enemy_hp_damage` is called but Invincible cap is not).
- **Fix:** Route retaliation through `deal_damage_to_enemy` / `calculate_incoming_damage` (type = THORNS). Add explicit `damage_type` tag to `deal_damage_to_enemy` so Curl Up / Malleable (NORMAL-only) are correctly gated off.

### DM5 — `tick_poison` (player path) bypasses source-power / source-relic hook chain (MEDIUM)
- **Symptom:** `powers/debuffs.rs:90` does `entity.hp -= damage` directly and writes new poison. Enemy poison tick goes through `record_enemy_hp_damage` which fires death hooks, but there is no source-side `onInflictDamage` from the card/relic that applied the poison. Player poison tick at `engine.rs:1529` pre-computes `apply_hp_loss(player_poison, intangible, tungsten_rod)` and then calls `player_lose_hp` — Intangible / Tungsten Rod handled correctly, but the `wasHPLost` chain (DM3) still misses.
- **Java:** `PoisonPower.atEndOfTurn` queues `DamageAction(owner, DamageInfo(source, poison, HP_LOSS))` which still calls `source.onInflictDamage` and target `wasHPLost`.
- **Impact:** Low for current content; matters for content that hooks poison-damage-dealt (Bottled Lightning variants, potential future relics).
- **Fix:** Emit `OnPoisonTick` event in tick_poison; emit `OnPlayerWasHPLost` (per DM3) in `player_lose_hp`. Keep Intangible/Tungsten correctly applied via `apply_hp_loss`.

### DM6 — `JUST_APPLIED` flag set without Java's `turnHasEnded && !source.isPlayer` gate (LOW)
- **Symptom:** `apply_debuff_from_enemy` in `powers/debuffs.rs:49` sets `*_JUST_APPLIED` flag on every successful enemy-sourced debuff application. Java `AbstractPower.justApplied` is only `true` when `source != null && !source.isPlayer && AbstractDungeon.actionManager.turnHasEnded`.
- **Impact:** Start-of-combat enemy power applications (e.g., turn 1 pre-move Weak) set `JUST_APPLIED` in Rust; Java would not, because `turnHasEnded == false` at start of combat. Results in Rust skipping the first decrement when Java would decrement as normal. Known scenarios:
  - Turn 1 openers that apply debuffs (Looter/Mugger turn-0 plan, etc.)
  - Enemy reactive applications during player's own turn (where `turnHasEnded == false`)
- **Fix:** Pass a `turn_has_ended: bool` flag into `apply_debuff_from_enemy`. Set only when called from enemy-turn execution context (`combat_hooks::do_enemy_turns`), not from reactive hooks triggered during the player turn.

### DM7 — Enemy block reset timing for Centurion/Protector (UNVERIFIED)
- **Symptom:** Enemy block is zeroed at start of that enemy's own turn in Rust. In Java, `AbstractMonster.applyStartOfTurnPowers` does NOT zero block; block is reset in `AbstractCreature.update()` before `takeTurn`. For enemies that act before/after another enemy (Centurion protecting Healer), block ordering matters.
- **Impact:** Centurion's Protect Healer sequence may drop block differently across turns vs Java.
- **Fix:** Verify Centurion start-of-turn sequencing test exists; if not, add.

### DM8 — `apply_invincible_cap` (unused variant) still live in codebase (LOW, cleanup)
- **Symptom:** Two Invincible implementations in `powers/debuffs.rs`: `apply_invincible_cap` (mutates cap) and `apply_invincible_cap_tracked` (separate per-turn tracker). Only `apply_invincible_cap_tracked` is used; `apply_invincible_cap` is dead code that would corrupt state if ever called.
- **Fix:** Delete `apply_invincible_cap` (lines 237–249). Not a parity bug, just latent bomb.

### DM9 — `modify_damage_receive` computes Slow+Intangible on `EntityState` but is never called on the hot path (LOW, cleanup)
- **Symptom:** `powers/debuffs.rs:140–155` defines `modify_damage_receive` returning `f64`. Greped callers: none in the engine. `deal_damage_to_enemy` has its own inline Slow + Flight + Invincible logic. Redundant.
- **Fix:** Delete `modify_damage_receive` or route `deal_damage_to_enemy` through it for consistency.

### DM10 — Source-power `onInflictDamage` not fanned out (MEDIUM, overlaps DM3)
- **Symptom:** No trigger exists for "this power's owner just inflicted damage." Used in Java for: `EnvenomPower` (player power, unlocks poison on NORMAL damage), `PhantasmalPower` (DoubleDamage decrement on hit), `VampirePower` (enemy lifesteal), `SpikerPower` (Anchor/gem equivalents).
- **Impact:** Envenom wired correctly via attack-hit path? Need to verify. Vampire: known missing.
- **Fix:** Add `Trigger::OnSourceInflictedDamage` (player) and `Trigger::OnEnemyInflictedDamage`. Fan out inside `deal_player_attack_hit_to_enemy` and `execute_enemy_move` respectively after HP damage resolves.

### DM11 — `deal_damage_to_enemy` Slow multiplier floors mid-computation (LOW)
- **Symptom:** Line 2476: `let damage_after_slow = (damage as f64 * slow_mult) as i32;`. Flight then multiplies by 0.5 on this already-floored integer. Java applies both multipliers on the same floating value before a single final floor.
- **Impact:** 9 damage vs Slow-1, Flight-1: Rust: `(9 * 1.1 = 9.9 → 9) → (9 * 0.5 = 4.5 → 4)` = 4. Java: `9 * 1.1 * 0.5 = 4.95 → 4`. Same result for most cases, but 5 vs Slow-1 Flight-1 is Rust `(5*1.1=5.5→5)*0.5=2.5→2`, Java `5*1.1*0.5=2.75→2`. Same. Edge case: 4 vs Slow-1 Flight-1: Rust `(4*1.1=4.4→4)*0.5=2`; Java `4*1.1*0.5=2.2→2`. No observable divergence found in quick samples; low-impact in practice.
- **Fix (optional):** Keep damage as `f64` through Slow and Flight, floor once. Not urgent.

### DM12 — Corpse Explosion routes through `deal_damage_to_enemy` but splash damage bypasses Pen Nib / STR (LOW)
- **Symptom:** `finalize_enemy_death` calls `deal_damage_to_enemy(other, max_hp)` which applies target Slow/Flight/Invincible/block. Java `CorpseExplosionPower` uses `DamageAction(DamageInfo(null, max_hp, THORNS))` — no attacker power chain. Rust matches via `deal_damage_to_enemy` which also skips attacker chain. OK.
- **Note:** This is a correct pass-through; logging here for audit completeness.

### DM13 — `player_lose_hp` effect hooks are event-based not ordered with Java (LOW)
- **Symptom:** `OnPlayerHpLoss` event fires BEFORE Rupture / Runic Cube / Fairy Revive checks. Java order is: (1) power `wasHPLost`, (2) relic `onLoseHp`, (3) revive check, (4) Rupture. Rust order: (1) OnPlayerHpLoss event (relic `onLoseHp` via this), (2) Rupture, (3) Runic Cube, (4) revive check.
- **Impact:** Rupture fires before revive, matching Java on practical combat outcomes. Runic Cube drawing before revive is OK since if player is dying the draw is moot. Low impact.
- **Fix:** None required.

### DM14 — `JUST_APPLIED` flag detection uses `entity.status(flag) > 0` not a set of flags per source-party (LOW)
- **Symptom:** `decrement_debuff_with_just_applied` reads a single flag per debuff type (e.g., `sid::WEAKENED_JUST_APPLIED`). If an enemy applies Weak to another enemy (rare but possible via Gremlin Leader mechanics, Looter run-away-blocking, etc.), the flag fires on the wrong target. More generally, `JUST_APPLIED` is per-entity in Java, tracked on the power instance itself, not a sid.
- **Impact:** Low for current content; correctness gap for enemy-debuffing-enemy scenarios.
- **Fix:** Track `just_applied` as a parallel status on the debuff target entity (already done — the flag lives on `EntityState`). Confirmed correct for current use cases; note in parity register as low-priority design difference.

## Items verified clean

- **Ginger/Turnip blocks** in `apply_debuff` at lines 200–207 correctly prevent Weak/Frail regardless of source. Matches Java `GingerPower.onApplyPower` / `TurnipRelic.onApplyPower`.
- **Artifact consumption** at line 192 is pre-Ginger/Turnip and matches Java `ArtifactPower.onApplyPower` priority.
- **Calipers decay** on player block handled correctly in `start_player_turn` per D27/D41 fix (not re-verified here — reference existing audit).
- **Turn 1 block preserve** D45 fix applied — start-of-combat gain_block before end_turn is correctly preserved.
- **Barricade** on both player and enemies blocks the end-of-turn block reset correctly per D43.
- **Blur** decrements at end of player turn not end of turn round — matches Java `BlurPower.atEndOfTurn`.
- **Retain / Ethereal** discard gate in `end_turn` correctly preserves Retain cards and exhausts Ethereal cards.
- **Innate draw sort** at start of combat matches Java.
- **Curl Up NORMAL-only gate** — implicit because `deal_damage_to_enemy` is the sole NORMAL path (THORNS / HP_LOSS use separate helpers). Correct per D63 fix noted in code comments.
- **Gremlin Nob enrage SKILL-only** — matches Java per existing audits.
- **Vault skipping end-of-round power pass** — D50 fix gates monster debuff decrement behind `!skip_enemy_turn`, matching Java's `if (!skipMonsterTurn)` gate.
- **Invincible tracker reset** at start of each enemy turn via `reset_invincible_damage_taken` call chain.
- **Just-applied clear** happens before decrement in `decrement_debuff_with_just_applied` (correct — Java `justApplied = false` at start of `atEndOfRound`).

## Follow-up questions

1. Should `deal_damage_to_player` and `combat_hooks::on_enemy_attack_player` be unified into a single `apply_incoming_damage(engine, raw, source_kind)` helper that both paths call, eliminating DM1 entirely?
2. Does `effects/runtime.rs:1715` `Target::Player` ever see damage already pre-multiplied by Vulnerable (via `calculate_damage_full`), or is it always raw? If always raw, DM1 fix is direct; if sometimes pre-multiplied, need to mark each callsite.
3. For DM2 (Flight double-application), enumerate which card effects go through `calculate_damage_full(flight)` vs `deal_damage_to_enemy` — is there a current observable damage bug in Byrds fights specifically?
4. Is there an appetite for adding a `damage_type: DamageType` enum passed through `deal_damage_to_enemy` (NORMAL / THORNS / HP_LOSS) to gate Curl Up / Malleable / Flight correctly? Currently implicit via call-path.
5. Should retaliation (DM4) be restructured so FlameBarrier / Thorns / Static Discharge / SharpHide all route through a single `retaliate(source, target, amount, type=THORNS)` helper?
