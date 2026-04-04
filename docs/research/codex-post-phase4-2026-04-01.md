# Post-Phase-4 Adversarial Review: Rust Engine vs Java Slay the Spire

**Date**: 2026-04-01
**Reviewer**: Claude Opus 4.6 (manual audit)
**Scope**: All files in `packages/engine-rs/src/`
**Method**: Line-by-line read of engine.rs, card_effects.rs, combat_hooks.rs, damage.rs, state.rs, status_keys.rs, orbs.rs, seed.rs, potions.rs, status_effects.rs, cards/*.rs, enemies/*.rs, powers/*.rs, relics/*.rs

---

## CRITICAL -- Wrong Behavior

### C1. Double Tap / Burst replay never fires in play_card
**File**: `engine.rs:1003-1010`
**Issue**: `replay_pending` field exists on `CombatState` (state.rs:261) but is never read or consumed by `play_card()`. The `consume_double_tap()` and `consume_burst()` functions exist in `powers/buffs.rs` but are never called from `engine.rs`. The only replay mechanism is EchoForm (line 1003), which replays only the *first* card. Double Tap (replay next Attack) and Burst (replay next Skill) are installed as status counters via `card_effects.rs:1251-1253` but never consumed to trigger a second play.

**Java**: `DoubleTapPower.onUseCard()` queues the card for replay. `BurstPower.onUseCard()` does the same for Skills.
**Impact**: Double Tap and Burst are completely non-functional. Playing these cards wastes energy and does nothing.

### C2. Thorns/Flame Barrier fire once per enemy attack instead of once per hit
**File**: `combat_hooks.rs:176-208`
**Issue**: Thorns and Flame Barrier are applied AFTER the multi-hit loop (lines 99-174), not inside it. In Java, Thorns deals its damage once per individual hit (multi-hit attacks like Sword Boomerang trigger Thorns for each hit). The Rust engine deals Thorns/Flame Barrier damage only once per enemy attack action, regardless of `move_hits`.

**Java**: `ThornsPower.onAttacked()` fires per hit. Each hit of a multi-hit attack triggers Thorns separately.
**Impact**: Thorns and Flame Barrier deal 1/N of correct damage against multi-hit enemies (e.g., Book of Stabbing, Reptomancer daggers).

### C3. Player Regeneration never triggers
**File**: `engine.rs` (end_turn function)
**Issue**: `apply_regeneration()` exists in `powers/enemy_powers.rs:149` and is used in `powers/buffs.rs:775` for the composite `EotResult` struct, but the engine's `end_turn()` method never calls any Regeneration trigger for the player. The Regen Potion correctly sets `"Regeneration"` status, but it never heals. Only the composite `compute_end_of_turn_effects()` function in `powers/buffs.rs` computes it, but that function is not called from `engine.rs`.

**Java**: `RegenerationPower.atEndOfTurn()` heals the player each turn and decrements.
**Impact**: Regen Potion and any Regeneration source is a no-op for the player. Enemies may also fail to regenerate if `compute_end_of_turn_effects()` is not wired.

### C4. Wave of the Hand never triggers Weak application
**File**: `engine.rs`, `card_effects.rs`
**Issue**: Wave of the Hand sets `WAVE_OF_THE_HAND` status (card_effects.rs:548), and it is reset at start of turn (engine.rs:192), but there is no code anywhere that checks this status to apply Weak when the player gains block. In Java, `WaveOfTheHandPower.onGainedBlock()` applies Weak to all enemies whenever block is gained.

**Java**: `WaveOfTheHandPower.onGainedBlock()` -> apply 1 Weak to all enemies per stack.
**Impact**: Wave of the Hand is installed but never triggers. The power card does nothing.

### C5. Spore Cloud on-death effect never triggers
**File**: `engine.rs` (check_combat_end and deal_damage_to_enemy)
**Issue**: `get_spore_cloud_vulnerable()` exists in `powers/enemy_powers.rs` (exported in mod.rs:1320), but when an enemy dies, neither `check_combat_end()` nor `deal_damage_to_enemy()` checks for SporeCloud to apply Vulnerable to the player. Fungi Beast has SporeCloud in Java.

**Java**: `SporeCloudPower.onDeath()` applies 2 Vulnerable to the player.
**Impact**: Fungi Beast deaths never debuff the player, making them easier than Java.

### C6. Blur never decrements -- persists forever
**File**: `engine.rs:209-210`
**Issue**: Blur is checked at start of turn (`self.state.player.status(sk::BLUR) > 0`) and causes block retention, but it is never decremented. In Java, Blur is turn-based and decrements each turn. The `decrement_debuffs` function handles Weakened/Vulnerable/Frail but Blur is a buff, not a debuff, so it never gets decremented.

**Java**: `BlurPower` is turn-based, decrements at end of turn via `atEndOfRound`.
**Impact**: Once Blur is gained (via Blur card or Wraith Form), it persists for the entire combat. Block never decays again.

---

## HIGH -- Missing Feature

### H1. Confusion/Snecko Eye cost randomization uses deterministic midpoint
**File**: `engine.rs:754-756`
**Issue**: `effective_cost()` (the `&self` version used for `can_play_card()`) returns a fixed cost of 1 for all cards under Confusion. While the `effective_cost_mut()` version correctly randomizes 0-3, the `can_play_card()` check uses the deterministic version, meaning cards that cost 0 in randomization will still show as "playable at cost 1" and cards that rolled 2-3 will also show as playable. This is explicitly marked as MCTS approximation but creates a real behavior gap.

**Java**: Each card's cost is randomized when drawn and displayed as that cost.
**Impact**: Under Snecko Eye, action generation is wrong -- it generates actions for cards that may actually cost more than available energy, and may miss 0-cost opportunities.

### H2. Enemy Regeneration (Darkling, Heart, etc.) not wired in enemy turn
**File**: `combat_hooks.rs:14-61`
**Issue**: `do_enemy_turns()` does not call `apply_regeneration()` for enemies. It handles poison, ritual, and metallicize, but enemy Regeneration (used by Darkling, Corrupt Heart when using Buff move) is never triggered.

**Java**: `RegenerationPower.atEndOfTurn()` fires for enemies too (Darklings heal each turn).
**Impact**: Darklings and any enemy with Regeneration never heal.

### H3. Invincible damage cap not enforced in deal_damage_to_enemy
**File**: `engine.rs:1320-1344`
**Issue**: `deal_damage_to_enemy()` handles Flight but does not check or enforce Invincible (damage cap per turn). The Heart, Donu, and Deca have Invincible (200-300 HP cap per turn). The `invincible_cap` function exists in `powers/debuffs.rs` but is never called from the damage pipeline.

**Java**: `InvinciblePower.wasHPLost()` caps total HP loss per turn.
**Impact**: The Heart and other Invincible enemies can be one-shot, completely breaking boss fight parity.

### H4. No DoubleDamage consumption in damage pipeline
**File**: `engine.rs`, `card_effects.rs`
**Issue**: The `double_damage` parameter is always passed as `false` in `calculate_damage_full()` calls (card_effects.rs:135, 178). `DoubleDamage` (from Phantasmal Killer, Double Damage potion) status exists but is never read or consumed in the card play path.

**Java**: `DoubleDamagePower.modifyDamageGive()` doubles outgoing attack damage, then decrements.
**Impact**: Phantasmal Killer and DoubleDamage effects do nothing.

### H5. Enemy Fading (die after N turns) not implemented
**File**: `combat_hooks.rs`
**Issue**: `Fading` status key exists (status_keys.rs:160) and is defined in `powers/mod.rs`, but `do_enemy_turns()` never checks or decrements it. Gremlins summoned by Gremlin Leader should die after 2 turns.

**Java**: `FadingPower.atEndOfTurn()` decrements and kills the enemy at 0.
**Impact**: Summoned gremlins persist indefinitely instead of dying after their fading turns.

### H6. Growth power not applied in enemy turns
**File**: `combat_hooks.rs`
**Issue**: `Growth` status key exists (status_keys.rs:162) but is never consumed in `do_enemy_turns()`. The Awakened One phase 2 and some enemies gain Strength/Block via Growth each turn.

**Java**: `GrowthPower.atEndOfTurn()` gives the enemy Strength and Block each turn.
**Impact**: Growth-based enemy scaling is missing, making those fights easier.

### H7. TheBomb countdown not ticking
**File**: `engine.rs`, `combat_hooks.rs`
**Issue**: `TheBomb` and `TheBombTurns` status keys exist but are never decremented or detonated. Bronze Automaton's orbs use TheBomb.

**Java**: `TheBombPower.atEndOfTurn()` decrements turns and deals massive damage at 0.
**Impact**: Bronze Automaton orbs with TheBomb never detonate, removing a major boss mechanic.

### H8. Slow power damage multiplier not applied
**File**: `engine.rs:971-975`
**Issue**: `increment_slow()` is called on card play, correctly tracking the counter. However, the Slow power's actual effect (10% more damage per stack to the enemy) is never applied in `deal_damage_to_enemy()` or `calculate_damage_full()`.

**Java**: `SlowPower.atDamageReceive()` adds 10% more damage per card played that turn.
**Impact**: Time Eater and other Slow enemies take normal damage instead of increasing damage per card played.

---

## MEDIUM -- Approximation OK for MCTS but Notable

### M1. Confusion always costs 1 in can_play_card (documented MCTS approximation)
**File**: `engine.rs:754`
Already covered in H1, but the `effective_cost_mut()` version does randomize correctly. The approximation in `can_play_card()` means legal action generation is imprecise under Snecko Eye.

### M2. Meditate returns cards from top of discard (not player-chosen)
**File**: `card_effects.rs:531-544`
`discard_pile.pop()` returns the last card added, not a player-selected card. In Java, Meditate lets the player choose which card(s) to return. For MCTS, this is acceptable since the agent optimizes over available actions.

### M3. Tools of the Trade discards random instead of player-chosen
**File**: `engine.rs:392-394`
Discards a random card instead of letting the player choose. Acceptable MCTS approximation.

### M4. Scry implementation discards from top of draw pile
**File**: Scry implementation uses simplified approach (look at top N, discard some). The Java version lets the player choose which to discard.

### M5. MummifiedHand grants 1 energy instead of making a random card cost 0
**File**: `engine.rs:1027-1029`
Documented approximation -- without per-card cost tracking, granting 1 energy is a reasonable substitute.

### M6. Creative AI / Hello World / Magnetism add placeholder cards
**File**: `engine.rs:284-308`
Creative AI adds "Smite" instead of a random Power. Hello World adds "Strike" instead of a random Common. Magnetism adds "Strike" instead of a random card. These are reasonable MCTS approximations.

### M7. Omniscience simplified to draw 2
**File**: `card_effects.rs:574-576`
Should play a card from hand twice for free. Drawing 2 is a rough approximation.

### M8. Wish simplified to gain Strength
**File**: `card_effects.rs:579-582`
Should offer choice (Plated Armor, Strength, or Gold). Strength-only is an acceptable default for MCTS.

---

## LOW -- Cosmetic / Minor

### L1. Potions use raw strings instead of sk:: constants
**File**: `potions.rs`
Multiple places use `"Strength"`, `"Weakened"`, etc. instead of `sk::STRENGTH`, `sk::WEAKENED`. Functionally correct since the string values match, but inconsistent with the codebase convention.

### L2. WELL_LAID_PLANS and RETAIN_CARDS are aliases
**File**: `status_keys.rs:67-68`
```rust
pub const RETAIN_CARDS: &str = "RetainCards";
pub const WELL_LAID_PLANS: &str = "RetainCards"; // alias
```
Both map to `"RetainCards"`. This works but is confusing -- Well-Laid Plans in Java limits retained cards to `amount` per turn, but the Rust engine treats it as unlimited retain.

### L3. Sadistic Nature key mismatch in PowerId
**File**: `status_keys.rs:106`, `powers/mod.rs:240`
`sk::SADISTIC` = `"SadisticNature"` but `PowerId::Sadistic.key()` = `"Sadistic"`. These are different strings, meaning any code using `PowerId::Sadistic.key()` to look up the status would fail. The engine uses `sk::SADISTIC` consistently so this is not currently triggered, but it is a latent bug.

### L4. Duplicate do_enemy_turns implementation
**File**: `engine.rs:1472-1499`, `combat_hooks.rs:14-61`
There is a `do_enemy_turns()` method on `CombatEngine` (engine.rs:1472) AND a standalone `do_enemy_turns()` in `combat_hooks.rs:14`. The `end_turn()` method calls `combat_hooks::do_enemy_turns(self)` (engine.rs:644), so the engine method is dead code. No behavioral impact but confusing.

### L5. RNG next_int uses modular reduction instead of rejection sampling
**File**: `seed.rs:103-106`
Comment says "rejection sampling for uniformity" but implementation uses `bits % bound`, which is modular reduction with slight bias. For MCTS this is negligible, but it does not match Java's `Random.nextInt(bound)` exactly.

---

## STATUS KEY AUDIT

### Defined but never consumed in engine:
- `SHIFTING` -- defined, never checked (Spire Shield power)
- `EXPLOSIVE` -- defined, never checked (Explosive enemy countdown)
- `GENERIC_STRENGTH_UP` -- defined, never checked
- `FORCEFIELD` -- defined in status_keys, `check_forcefield()` exists in powers but never called from engine
- `MALLEABLE` -- defined, never consumed (Bronze Automaton power)
- `REACTIVE` -- defined, never consumed
- `SKILL_BURN` -- defined, never consumed (Heart mechanic)
- `TIME_WARP_ACTIVE` -- defined, usage unclear vs `TIME_WARP`

### Consumed correctly:
- `STRENGTH`, `DEXTERITY`, `FOCUS`, `VIGOR` -- all wired
- `VULNERABLE`, `WEAKENED`, `FRAIL`, `POISON` -- all wired
- `INTANGIBLE`, `BUFFER`, `FLIGHT` -- all wired in damage pipeline
- `BARRICADE`, `METALLICIZE`, `PLATED_ARMOR` -- all wired
- `THORNS`, `FLAME_BARRIER` -- wired but per-attack not per-hit (see C2)
- `BEAT_OF_DEATH`, `CURIOSITY`, `ENRAGE`, `ANGRY` -- all wired
- `ENTANGLED`, `CONFUSION`, `CORRUPTION` -- all wired in can_play_card/effective_cost
- `MENTAL_FORTRESS`, `RUSHDOWN`, `DEVOTION` -- all wired in change_stance/start_player_turn
- `TIME_WARP`, `SLOW` -- increment wired, but Slow damage bonus not applied (H8)

---

## SUMMARY BY COUNT

| Severity | Count | Examples |
|----------|-------|---------|
| CRITICAL | 6 | DoubleTap/Burst dead, Thorns per-attack not per-hit, Regen never fires, Wave of Hand dead, SporeCloud dead, Blur infinite |
| HIGH | 8 | Invincible cap missing, Fading dead, Growth dead, TheBomb dead, DoubleDamage dead, Slow damage bonus, Enemy Regen, Confusion approximation |
| MEDIUM | 8 | Various MCTS approximations (documented) |
| LOW | 5 | String inconsistencies, dead code, minor RNG bias |

**Recommendation**: Fix all CRITICAL issues first -- they represent completely broken game mechanics. Then address HIGH issues which remove important enemy abilities (Invincible, Fading, Growth, TheBomb), making the engine significantly easier than Java Slay the Spire.
