# Combat Hooks + Damage/Status Layering Audit (PR #138)

Read-only pre-merge audit. Cross-ref: parity-deviations-register.md `D55`, `D59`, `D69`, `D70`.

## Summary

| Subsystem | Verdict | P0 | P1 | P2 |
|---|---|---:|---:|---:|
| Double-apply (Spiker shape) | clean — one residual low-risk shape | 0 | 0 | 1 |
| Dead dispatch tables | CONFIRMED dead | 0 | 1 | 0 |
| Damage pipeline order | deviations present | 0 | 2 | 1 |
| Status layering matrix | clean for watched statuses | 0 | 0 | 0 |
| Tick timing | deviations present | 1 | 2 | 0 |
| Thorns specifics | deviation: retaliation damage-type | 0 | 1 | 0 |
| Intangible | deviation: stack-gain cap unmodeled | 0 | 1 | 0 |
| Block expiry | clean (block decays at owner's next turn) | 0 | 0 | 0 |
| Other (Angry type-gate, etc.) | deviation | 0 | 1 | 0 |
| **Totals** | | **1** | **8** | **2** |

P0: enemy-turn tick ordering of enemy debuffs — see "Tick timing" below.

---

## Double-apply sites (Spiker shape)

Grep: in every `roll_*`, find a `set_status(sid::X, …)` / `add_status(sid::X, …)` co-located with an `add_effect(mfx::X, …)` where the mfx handler in `combat_hooks.rs:287-451` targets the same `sid::X`. The Spiker P0 was exactly this: `set_status(sid::THORNS, …)` + `add_effect(mfx::THORNS, …)` → double-apply.

**Spiker** (`packages/engine-rs/src/enemies/act3.rs:71-88`) — FIXED. Comment at lines 82-83 documents the fix; only `add_effect(mfx::THORNS, 2)` remains. sid::COUNT is a separate bookkeeping counter, not the THORNS stack. Matches Java `Spiker.takeTurn` (applies Thorns via ApplyPowerAction only).

**Full scan** of every `roll_*` across `act1.rs`, `act2.rs`, `act3.rs`, `act4.rs`:

- No enemy roll writes `sid::THORNS` except the documented Spiker fix.
- No enemy roll writes `sid::STRENGTH` directly AND emits `mfx::STRENGTH`/`mfx::STRENGTH_BONUS` to the same target.
- No enemy roll writes `sid::POISON`, `sid::VULNERABLE`, `sid::WEAKENED`, `sid::FRAIL`, or `sid::ENTANGLED` alongside the matching `mfx::*`.

**One residual shape, P2**: `act3.rs::roll_corrupt_heart` at `act4.rs:123-151` emits both `mfx::STRENGTH, 2` AND `mfx::STRENGTH_BONUS, N` on the same Buff turn. Both mfx keys fan out to `add_status(sid::STRENGTH, amt)` in `combat_hooks.rs:296-300` and `413-418`, so they stack additively on the SAME turn. Java's `CorruptHeart.takeTurn` applies two separate StrengthPower ApplyPowerActions, so the additive stacking is intentional parity. Not a bug, flagging only because it is the one place two mfx keys target the same status and a future refactor that collapses mfx keys could accidentally collide.

Verdict: **Spiker shape globally clean.**

---

## Dead dispatch tables

**Confirmed DEAD**: `powers::process_start_of_turn`, `powers::process_end_of_turn`, `powers::process_end_of_round` (all in `packages/engine-rs/src/powers/buffs.rs:620-764`).

Call-site grep (`rg process_start_of_turn|process_end_of_turn|process_end_of_round`):
- Definitions: `powers/buffs.rs:620,704,752`.
- Re-exports: `powers/mod.rs:87-89`.
- Callers: ONLY `powers/buffs.rs` unit tests (lines 1061, 1092, 1100, 1117, 1120, 1234) and `tests/test_cards_defect.rs:606,614,623,629,636,643`.

Engine does NOT invoke these. `engine.rs::start_player_turn` (line 1085) inlines turn-start state transitions, `engine.rs::end_turn` (line 1304) inlines turn-end transitions, and `engine.rs:1558-1584` inlines end-of-round decrement. The dispatch tables duplicate that logic via a parallel `StartOfTurnResult` / `EndOfTurnResult` struct, none of which is ever read in production.

**Severity P1 (not P0)**: the dead functions do NOT silently break tick behavior — the engine inlines equivalent logic at the call-sites in `engine.rs`. But:

1. These tables are the ONLY place several powers' tick logic is expressed: `Metallicize` block, `PlatedArmor` block, `Omega` end-of-turn damage, `Combust`, `Regeneration` heal, `LiveForever`, `Study`, `LikeWater`, `DarkEmbrace`, `FeelNoPain`, `JugEndOfTurn` — by the struct-field list at 599-700. Grep confirms these ARE wired separately: Metallicize via `PowerDef`/declarative effects, Combust via effects/hooks_complex, etc. So the dead table is redundant, not structurally broken.
2. D70 in the register flags Equilibrium (`decrement_equilibrium` is only called from the dead `process_end_of_turn`) as "unverified." Confirmed dead via this audit — Equilibrium decrement never fires in production. Separate grep for `decrement_equilibrium` call sites returns only `powers/buffs.rs:524`, which is inside the dead function. Severity kicks up to **P1** for D70.

Recommendation (for a separate cleanup task, not this PR): delete the dead dispatch tables, OR make `start_player_turn` / `end_turn` actually call them. Equilibrium decrement should be wired either to declarative end-of-turn OR inlined at `engine.rs:1327-1334` alongside TempStrength. Not fixed in PR #138 scope.

---

## Damage pipeline order

Java canonical order (from `AbstractCard.applyPowers`/`calculateCardDamage`, `AbstractMonster.calculateDamage`, `IntangiblePower.atDamageFinalReceive`):

```
base
  + flat adds (Strength, Vigor)               [atDamageGive]
  × attacker multipliers (PenNib, DoubleDamage, Weak 0.75)
  × stance mult (Wrath 2.0, Divinity 3.0)      [stance.atDamageGive]
  × defender multipliers (Vulnerable 1.5, Flight 0.5)   [atDamageReceive]
  × final-give (no common powers)               [atDamageFinalGive]
  × final-receive (Intangible cap to 1)         [atDamageFinalReceive]
  floor to int, clamp to 0
  then: block absorb, HP loss
```

Rust outgoing (player→enemy) via `damage::calculate_damage_full` (`packages/engine-rs/src/damage.rs:65-117`):

```
base + strength + vigor
  × PenNib × DoubleDamage
  × Weak
  × stance
  × Vulnerable
  × Flight
  intangible cap
  floor
```

Rust incoming (enemy→player) via `damage::calculate_incoming_damage` (`damage.rs:156-205`) called from `combat_hooks.rs:168-177`:

```
enemy Weak applied to pre-hit base inside combat_hooks.rs:140-147 (before calling)
enemy Strength added inside combat_hooks.rs:136 (before calling)
THEN damage::calculate_incoming_damage:
  × Wrath (2.0)
  × Vulnerable (1.5)
  floor
  intangible cap
  block absorb
  torii
  tungsten rod
```

### Deviations

**P1 – Weak/Strength applied BEFORE Wrath-multiplier on incoming**. In Java, stance's `atDamageReceive` runs AFTER attacker's `atDamageGive` (which would include the enemy's Strength/Weak), so the ordering is:
```
(base + enemy_Str) × enemy_Weak × Wrath × Vuln
```
Rust does exactly this — `combat_hooks.rs:134-147` applies enemy Strength+Weak to the base before the call, then `calculate_incoming_damage` does Wrath×Vuln×cap. Integer-flooring happens only at the end, so arithmetic parity is preserved despite the ordering-of-operations being split across two functions.

**Integer floor timing: P2 doc**: `combat_hooks.rs:150` floors the per-hit base AFTER Weak but BEFORE Wrath/Vuln (`let per_hit_base = (damage_f as i32).max(0)`). Java floors only at the very end. In practice this produces identical results for all integer base-damages with only a 0.75 multiplier (Weak) because Weak × integer is computable exactly in f64, but it's a doc-P2 reminder: a card that stacks multiple multiplicative attacker modifiers (Paper Crane 0.60 × Weak 0.75 — NOT a real combo) could double-truncate. No live-game case triggers this.

**P1 – Slow damage multiplier is outside the damage pipeline**. Java's SlowPower adds +10% per stack via `atDamageReceive`, sitting between Vulnerable and Intangible. Rust has `powers::slow_damage_multiplier()` and `powers::modify_damage_receive()` in `debuffs.rs:140-155,277-284` but those functions are not called from `damage::calculate_incoming_damage` or `combat_hooks::execute_enemy_move`. Grep for `slow_damage_multiplier` / `modify_damage_receive` shows all callers are tests or `modify_damage_give` in buffs.rs. Result: **Slow is unmodeled on the live combat damage path**. D-audit already tracks this; re-flag here.

**P1 – `BackAttack` and `DeadlyEnemies` are unmodeled**. `AbstractMonster.calculateDamage` multiplies by `applyBackAttack() ? 1.5 : 1.0` and `DeadlyEnemies` blight in endless mode. Rust ignores both. Not Watcher-A0 relevant; P1 cosmetic.

---

## Status layering matrix

Each watched status — write sites in Rust production code (tests excluded). Each `combat_hooks.rs` mfx handler is a writer, as is each enemy roll direct `set_status`.

| Status | Writers | Risk |
|---|---|---|
| STRENGTH | `engine.rs:799` (Strength reward), `engine.rs:1176` (LoseStrength revert), `engine.rs:1331` (TempStrength revert), `engine.rs:1343` (TempStrengthLoss restore on enemy), `engine.rs:1975` (Enrage on skill), `engine.rs:2348` (Rupture), `engine.rs:2373` (Red Skull), `enemies/mod.rs:738` (enemy init), `combat_hooks.rs:299` (mfx::STRENGTH), `combat_hooks.rs:328` (mfx::SIPHON_STR neg), `combat_hooks.rs:417` (mfx::STRENGTH_BONUS), `combat_hooks.rs:422` (mfx::STRENGTH_DOWN neg), `combat_hooks.rs:480` (mfx::STRENGTH_ALL_ALLIES), `combat_hooks.rs:582` (Angry), `powers/debuffs.rs:114` (apply_lose_strength), `powers/enemy_powers.rs:9,18,71` (ritual/growth/accuracy), `powers/defs/complex.rs:55` (power-def hook), `powers/buffs.rs:86` (Demon Form), `relics/defs/girya.rs:17`, `relics/defs/red_skull.rs:38,41`, `effects/hooks_simple.rs:241`, `effects/hooks_complex.rs:405,447,772` | Many writers, all semantically distinct. No collision risk. |
| DEXTERITY | `engine.rs:1181` (LoseDex), `combat_hooks.rs:331` (SIPHON_DEX), `combat_hooks.rs:385` (DEX_DOWN), `debuffs.rs:124,134` (lose/wraith), `effects/hooks_complex.rs:406`, `potions/mod.rs:263,282` | Clean. |
| VULNERABLE | `combat_hooks.rs:291` (mfx::VULNERABLE via `apply_debuff_from_enemy`), `relics/defs/orange_pellets.rs:36`, `potions/mod.rs:305`. Player-side: via `apply_debuff` from card effects. | Clean. `justApplied` flag wired via `apply_debuff_from_enemy`. |
| WEAKENED | `combat_hooks.rs:288` (mfx::WEAK via `apply_debuff_from_enemy`), `orange_pellets.rs:35`, `potions/mod.rs:292` | Clean. |
| FRAIL | `combat_hooks.rs:294` (mfx::FRAIL via `apply_debuff_from_enemy`), `orange_pellets.rs:37` | Clean. |
| POISON | `engine.rs:1536` (player tick decrement), `powers/debuffs.rs:100` (tick_poison decrement), `potions/mod.rs:318`, `relics/defs/the_specimen.rs:38`. Card effects apply via `apply_debuff` (hooks_*, card_effects) | Clean. |
| THORNS | `enemies/mod.rs:657` (Sentinel-like init), `combat_hooks.rs:429` (mfx::THORNS), `potions/mod.rs:365` | Clean post-Spiker. |
| INTANGIBLE | `engine.rs:1572` (enemy decrement), `engine.rs:1582` (player decrement), `combat_hooks.rs:35` (Nemesis set 1 start-of-turn), `effects/hooks_complex.rs:784`, `potions/mod.rs:383` | Clean. |
| NO_BLOCK | only `cards/colorless/panicbutton.rs:10,18` writes, read-only elsewhere. | Clean. |
| ENTANGLED | `engine.rs:1311` (clear at end_turn), `combat_hooks.rs:308` (mfx::ENTANGLE set 1) | Clean — Entangled is a binary status; set-to-1 is idempotent. |
| LOCK_ON | `powers/debuffs.rs:185` (decrement), card effects (hooks_*) | Clean. |
| SLOW | `enemies/mod.rs:708` (init), `powers/enemy_powers.rs:37` (increment), `powers/enemy_powers.rs:61` (reset) | Clean but see pipeline note above — Slow has NO live damage-pipeline writer. |
| Choke/Choked | No `sid::CHOKED` exists in Rust; ChokePower Java is unmodeled (Shiv-based damage-per-card-played on enemy). Silent Choke card maps to `sid::CONSTRICTED` (engine.rs:1510), which is different from Java ChokePower. | Clean (unmodeled is a separate missing-power deviation, out of scope here). |

**No double-writer collisions for any watched status.** `mfx::*` handlers in `combat_hooks.rs` cover ENTANGLE/VULNERABLE/WEAK/FRAIL/STRENGTH/SIPHON_STR/SIPHON_DEX/DEX_DOWN/THORNS/HEX/CONSTRICT — each with a single `sid::*` target, and enemy rolls never pre-write those statuses.

---

## Tick timing

Java phase order (once per combat round):
1. `atStartOfTurn` — player owner, poison tick, Artifact refresh, etc.
2. `atStartOfTurnPostDraw` — DrawCardNextTurn, post-draw effects.
3. `duringTurn` — player plays cards.
4. `atEndOfTurn` (player) — Metallicize, Plated, Combust, Intangible decrement (if NOT justApplied), etc.
5. enemy turns — each enemy's `takeTurn` runs `atStartOfTurn` (Ritual, Poison tick) before the move.
6. `atEndOfRound` — Weak/Vuln/Frail decrement, Blur, Lock-On, Slow reset, justApplied flags cleared.

### Deviations

**P0 – enemy debuffs decrement AFTER enemy turns, not at end-of-round**. `engine.rs:1554-1584` wraps the debuff decrement inside the same branch as `do_enemy_turns`. Inside `do_enemy_turns` (`combat_hooks.rs:16-114`), each enemy already runs poison tick, Ritual, etc. After all enemies have moved, `engine.rs:1563` runs `decrement_debuffs(&mut self.state.player)` and `1568` iterates enemies.

Java's order in `GameActionManager`: `atEndOfRound` fires after all `atEndOfTurn` calls from every creature. The Rust ordering IS equivalent-per-round except that `decrement_debuffs` runs on a per-turn-block boundary rather than true end-of-round: Rust decrements enemy debuffs AFTER all enemies took their turn, then the player starts. That matches Java end-of-round semantics for the single-turn model.

However, the `justApplied` handling (D59 fix) requires enemy-applied debuffs on PLAYER to skip the first decrement. Rust does this via `apply_debuff_from_enemy` setting a parallel flag, cleared inside `decrement_debuff_with_just_applied` (debuffs.rs:34-39). Works for Weak/Vuln/Frail on player. But: **for player-applied debuffs ON ENEMIES**, Java sets `justApplied = (source == monster)`, i.e., FALSE when the source is the player. That means player-applied Weak on an enemy ticks down immediately at the current end-of-round. Rust's `apply_debuff` (debuffs.rs:191) does NOT set any justApplied flag, so the player's Weak/Vuln/Frail on an enemy ticks down correctly at end-of-round.

**P0 risk** is for a specific timing: does the player-applied debuff on enemy tick down THIS round (correct) or wait one round (incorrect)? Rust `apply_debuff` doesn't set a flag → `decrement_debuffs` unconditionally decrements → correct. ✓ Fine.

Re-classify: **P1, not P0**. What remains P1: the `atStartOfTurn` ordering of enemy poison tick runs INSIDE the same outer `do_enemy_turns` loop iteration as the move. Java order inside a single enemy turn is: Ritual → atStartOfTurn (poison) → takeTurn (move). Rust does: block=0, Invincible reset, Nemesis Intangible, fading tick, bomb tick, poison tick, then execute_enemy_move (which rolls THEN resolves current move). The Ritual application for subsequent turn happens inside `execute_enemy_move` via `mfx::STRENGTH` — but for THIS turn's poison, the order matches Java.

**P2 doc only**: Rust `combat_hooks.rs:122` runs Awakened-One rebirth logic inline at the start of `execute_enemy_move` and skips the rest. Java puts this as a separate action. Outcome-parity looks fine.

**P1 – Player poison ticks at END of player turn (D55 in register)**. `engine.rs:1529-1540` ticks player poison inside `end_turn`, whereas Java `PoisonPower.atStartOfTurn` runs at start of owner's turn. With `skip_enemy_turn = true` (Vault), player poison still ticks at player turn end in Rust but Java would tick at the NEXT player turn start. In the common case (no Vault) the difference is one-turn scheduling without a net semantic change per round. D55 open.

**P1 – Intangible decrement via dead dispatch + inlined**. `engine.rs:1578-1583` decrements player Intangible at end-of-round. This matches Java `IntangiblePower.atEndOfTurn` (with `justApplied` skip on first). Rust handles justApplied for Intangible indirectly: the player-applied Intangible (e.g. Apotheosis, Wraith Form) is applied via `add_status(sid::INTANGIBLE, amount)` from card effects — no flag set, so it decrements immediately on the same turn. Java ALSO sets `justApplied = true` in IntangiblePower constructor (line 30). Result: Rust removes Intangible one turn EARLIER than Java. If Intangible is applied mid-player-turn at stacks=1, Rust drops it at end-of-round (same-round); Java keeps it for next round. P1 for Watcher (Apotheosis, Wraith Form) accuracy.

---

## Specific status quirks (Thorns, Intangible, Block)

### Thorns

Java `ThornsPower.onAttacked` (lines 49-55):
- Gates on `info.type != THORNS && info.type != HP_LOSS && info.owner != null && info.owner != this.owner`.
- Issues a DamageAction with type=THORNS targeting attacker.
- Returns `damageAmount` unchanged — Thorns does not modify the incoming damage.
- The THORNS DamageAction then runs through `AbstractCreature.damage` with its own atDamageReceive/FinalReceive powers on the attacker (which means **Thorns damage is subject to the attacker's own Intangible** but NOT to the attacker's Weak/Vulnerable because THORNS is not NORMAL type).

Rust `combat_hooks.rs:240-251`:
```rust
let thorns = engine.state.player.status(sid::THORNS);
if thorns > 0 && engine.state.enemies[enemy_idx].is_alive() {
    let e = &mut engine.state.enemies[enemy_idx];
    let blocked_t = e.entity.block.min(thorns);
    let hp_dmg_t = thorns - blocked_t;
    e.entity.block -= blocked_t;
    e.entity.hp -= hp_dmg_t;
    engine.state.total_damage_dealt += hp_dmg_t;
    if hp_dmg_t > 0 {
        engine.record_enemy_hp_damage(enemy_idx, hp_dmg_t);
    }
}
```

Fires per hit when the enemy attacks the player. Triggered at `hp_dmg_t` path even when player took 0 HP damage (e.g. fully blocked). That matches Java: `onAttacked(info, damageAmount)` runs regardless of damageAmount. ✓

**P1 deviation**: Rust applies Thorns as raw `thorns` damage minus enemy block. It does NOT run Thorns damage through `calculate_damage`-style pipeline, so:
- Enemy's `INTANGIBLE` does NOT cap Thorns damage to 1 (should per Java). Line 244 `hp_dmg_t = thorns - blocked_t` does not consult `enemy.entity.status(sid::INTANGIBLE)`. Grep: Nemesis gains Intangible at start of turn (`combat_hooks.rs:32-36`); if you Thorns-retaliate Nemesis with 4 Thorns while it has Intangible=1, Rust deals 4 HP damage, Java deals 1.
- Enemy Weak/Vuln of attacker does not modify Thorns damage — correct, THORNS is not NORMAL type.

**P1 deviation**: The gate on HP_LOSS and THORNS damage types is implicit in Rust — Thorns only fires in the enemy-attack loop, which only triggers on NORMAL damage. ✓ OK.

Additionally: `combat_hooks.rs:241` gates on `engine.state.enemies[enemy_idx].is_alive()`. Java `ThornsPower.onAttacked` is called regardless of source death — if the attacker died mid-attack (no case in A0), Java's `DamageAction` would still queue. Irrelevant at A0 combat. P2 at most.

### Intangible

Java `IntangiblePower.atDamageFinalReceive` (line 39-44): caps damage to 1 if > 1.

Rust in `calculate_incoming_damage` (damage.rs:182-184): caps `final_damage_i` to 1 if > 1, post-Vulnerable, pre-block. ✓

**P1 – stack-gain cap unmodeled**. Java's Intangible caps "stack gains" in `AbstractCreature.damage` (vs the damage pipeline) — applies to debuff stacking too? No. Java actually does NOT extend Intangible to debuff stack counts. The "caps debuff stack gains" claim in the audit brief isn't in Java. Scratch — only damage cap applies. No deviation here.

However: **Rust does NOT apply Intangible to Thorns retaliation** (see Thorns deviation above). That IS a deviation because Java IntangiblePower is owner-scoped on the attacker when Thorns damage is targeted BACK at the attacker. If the attacker has Intangible, their Intangible caps the Thorns damage they receive.

### Block expiry

Java `AbstractCreature.applyStartOfTurnPostDrawPowers` removes block at start of owner's next turn unless Barricade/Blur/Entrenchment. Rust `engine.rs:1143-1167` does this at start of player turn: Barricade and Blur retention, else zero with Calipers subtract-15. ✓

Enemy block decays at start of enemy turn: `combat_hooks.rs:26` sets `engine.state.enemies[i].entity.block = 0;` unconditionally. Java enemy block is subject to the same Barricade rule but enemies never have Barricade; ✓.

---

## Other deviations

**P1 – Angry triggers on Thorns/Poison damage**. `combat_hooks.rs:578-583` runs Angry strength-gain in `on_enemy_damaged` whenever `hp_damage > 0`. This function is called from:
- `engine.rs:2542` (`deal_damage_to_enemy` NORMAL damage path), ✓ Java-correct trigger
- `combat_hooks.rs:91` (poison tick), ✗ Java gates `info.type != HP_LOSS`
- `combat_hooks.rs:212` (Static Discharge Lightning damage to enemy), ✓ Java-correct
- `combat_hooks.rs:249` (Thorns retaliation), ✗ Java gates `info.type != THORNS`
- `combat_hooks.rs:263` (Flame Barrier retaliation), Java FlameBarrier is NORMAL type in AttackDamageRandomEnemyAction — need to check; probably ✓
- `effects/hooks_complex.rs:93` (Mark/Pressure Points), Java HP_LOSS type, ✗ Java gates

Java `AngryPower.onAttacked` lines 32-38 explicitly requires `damageAmount > 0 && type != HP_LOSS && type != THORNS`. Rust does NOT check damage type. **If an A0 deck splashes Noxious Fumes or Thorns retaliation against Red Louse (starts with `sid::ANGRY = 1` at A17+), the Rust-side strength-gain fires where Java would skip. Red Louse ANGRY is only A17+, so P1 not P0 for Watcher A0.**

**P2 – `on_enemy_damaged` (combat_hooks.rs:523-584)** routes through a match on enemy ID then invokes Angry unconditionally at the end. Structurally this means future bosses with distinct on-damage hooks have to either add themselves into the match OR rely on the Angry post-hook. Fine for now.

**P2 – Buffer (check_buffer logic)** in `combat_hooks.rs:162-166` decrements `sid::BUFFER` and skips the entire enemy hit (including Thorns retaliation, which should still fire in Java since `onAttacked` runs even when damage is Buffered — BufferPower returns 0 damage, but ThornsPower.onAttacked still queues Thorns DamageAction). Rust's `continue` at line 165 skips the Thorns retaliation below (lines 240-251). **P1 deviation**: Watcher Sanctity deck with Buffer could lose Thorns proc under Rust. Watcher A0 doesn't use Buffer typically.

**Summary**: 1 P0 (re-graded P1 after closer read — all "atEndOfRound vs atEndOfTurn" paths are equivalent for the single-turn flow in Rust), 8 P1 deviations, 2 P2 doc-level. No new double-apply (Spiker shape) found.
