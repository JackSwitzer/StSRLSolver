# Damage Pipeline Audit: Python Engine vs Decompiled Java

Audit date: 2026-02-02

## Java Damage Chain (Source of Truth)

### Outgoing Damage: `DamageInfo.applyPowers(owner, target)`

**Player attacking enemy (owner.isPlayer == true):**
1. Owner powers `atDamageGive` (Strength +flat, Vigor +flat, Weak *0.75, Pen Nib *2.0)
2. Stance `atDamageGive` (Wrath *2.0, Divinity *3.0)
3. Target powers `atDamageReceive` (Vulnerable *1.5, Slow *1.0+0.1*stacks)
4. Owner powers `atDamageFinalGive` (none relevant for Watcher)
5. Target powers `atDamageFinalReceive` (Intangible: cap 1, Flight: /2.0)
6. `MathUtils.floor(tmp)`, min 0

**Enemy attacking player (owner.isPlayer == false):**
1. Owner powers `atDamageGive` (Strength +flat, Weak *0.75 or *0.6 with Paper Crane)
2. Target(player) powers `atDamageReceive` (Vulnerable *1.5 or *1.25 with Odd Mushroom)
3. Stance `atDamageReceive` (Wrath *2.0)
4. Owner powers `atDamageFinalGive`
5. Target powers `atDamageFinalReceive` (IntangiblePlayer: cap 1)
6. `MathUtils.floor(tmp)`, min 0

### Incoming Damage: `AbstractPlayer.damage(DamageInfo info)`

After `applyPowers` computes `info.output`:
1. IntangiblePlayer check: if `damageAmount > 1`, set to 1 (redundant with applyPowers)
2. `decrementBlock(info, damageAmount)` - block absorbs damage
3. `onAttackedToChangeDamage` - relics then powers (post-block modifications)
4. `onAttacked` - powers then relics (Torii: if `> 1 && <= 5`, set to 1)
5. `onLoseHpLast` - relics (Tungsten Rod: -1)
6. Subtract from HP

**Critical ordering: Torii and Tungsten Rod are applied AFTER block removal.**

### HP_LOSS Damage Type

HP_LOSS bypasses block entirely. Torii explicitly excludes HP_LOSS. Tungsten Rod applies via `onLoseHpLast` (applies to all damage types). Intangible applies to all types in `atDamageFinalReceive`.

## Python Implementation Analysis

### `packages/engine/calc/damage.py`

**`calculate_damage()` - Outgoing damage:**
- Order: base -> +Strength +Vigor -> *Pen Nib -> *DoubleDamage -> *Weak -> *Stance -> *Vuln -> *Flight -> Intangible cap -> floor, min 0
- All computation in float, single floor at the end

**`calculate_incoming_damage()` - Incoming damage to player:**
- Order: *Wrath -> *Vuln -> floor -> Intangible cap -> Torii -> block -> Tungsten Rod

**`apply_hp_loss()` - HP_LOSS type:**
- Intangible cap -> Tungsten Rod

### `packages/engine/combat_engine.py`

**`_calculate_card_damage()`**: Calls `calculate_damage(vuln=False)` - deliberately excludes Vuln.
**`_deal_damage_to_enemy()`**: Applies Vuln separately: `int(damage * VULN_MULT)`.

## Discrepancies Found

### BUG 1 (Critical): Rounding Error from Split Vuln Application

**Location:** `combat_engine.py` lines 1489-1514, 1262-1267

**Problem:** The combat engine splits the damage calculation into two steps:
1. `_calculate_card_damage()` computes `int(float_chain)` WITHOUT Vuln
2. `_deal_damage_to_enemy()` applies Vuln: `int(result * 1.5)`

Java computes everything in a single float chain and floors ONCE at the end.

**Example:** Strike(6) + 3 Str + Weak + Vuln target
- Java: `(6 + 3) * 0.75 * 1.5 = 10.125 -> floor -> 10`
- Python: `int((6+3) * 0.75) = int(6.75) = 6`, then `int(6 * 1.5) = 9`
- **Result: Python=9, Java=10. Off by 1.**

**Fix:** Pass `vuln=True` into `calculate_damage()` and remove the separate vuln application in `_deal_damage_to_enemy()`.

### BUG 2 (Critical): Torii Applied BEFORE Block

**Location:** `calc/damage.py` `calculate_incoming_damage()` lines 266-269

**Problem:** Python applies Torii before block subtraction. Java applies Torii AFTER block removal (via `onAttacked` hook, which runs after `decrementBlock`).

**Example:** 4 damage, 3 block, Torii
- Java: 4 - 3 block = 1 unblocked. Torii doesn't trigger (1 is not > 1). HP loss = 1.
- Python: Torii triggers (2 <= 4 <= 5) -> damage = 1. 1 - 1 block = 0 HP loss. **Wrong: player takes 0 instead of 1.**

**Example:** 4 damage, 0 block, Torii
- Java: 4 - 0 block = 4. Torii triggers (4 > 1 && 4 <= 5) -> 1. HP loss = 1.
- Python: Torii triggers (2 <= 4 <= 5) -> 1. 1 - 0 block = 0... wait, 1 - 0 = 1. Same result.
- **But with partial block, results diverge.**

**Fix:** Move Torii check to after block subtraction, operating on `hp_loss` not `final_damage`.

### BUG 3 (Minor): Torii Lower Bound Check

**Location:** `calc/damage.py` line 268

**Problem:** Python checks `2 <= final_damage <= 5`. Java checks `damageAmount > 1 && damageAmount <= 5`.
For integers, `> 1` is equivalent to `>= 2`, so this is technically correct. But Torii in Java also checks `info.owner != null` and `info.type != HP_LOSS && info.type != THORNS`. The Python function doesn't track damage type, but since `calculate_incoming_damage` is only called for NORMAL damage, this is implicitly correct.

**Status:** Not a bug, but the Torii check should document the type assumption.

### BUG 4 (Minor): Intangible Double-Application

**Location:** `calc/damage.py` lines 147 and 262-264

**Problem:** Python applies Intangible in both `calculate_damage()` (outgoing) and `calculate_incoming_damage()` (incoming). Java also applies it in both `applyPowers` and `AbstractPlayer.damage`. Since capping at 1 is idempotent, this doesn't cause incorrect results, but it means the Python `calculate_damage` function applies Intangible at the wrong conceptual stage (it's on the receiver, not the attacker's output calculation).

**Status:** Functionally correct but conceptually misleading.

### BUG 5 (Moderate): Pen Nib vs Weak Ordering

**Location:** `calc/damage.py` lines 121-132

**Problem:** Python applies Pen Nib before Weak. In Java, both are `atDamageGive` hooks, and iteration order depends on power insertion order. Typically Strength is added at game start, and Pen Nib triggers every 10th attack. The actual Java order within `atDamageGive` loop is: iterate `owner.powers` list once. If Weak is added before Pen Nib in the list, Weak applies first.

However, since both are multiplicative on the same float, the order doesn't affect the result (multiplication is commutative). The floor only happens once at the end. **Not actually a bug.**

**Status:** Not a bug (commutative multiplication).

### Observation: `effects/registry.py` Damage Functions

The `EffectContext._apply_damage_to_enemy()` (line 266) does NOT apply Vulnerable. It just does block subtraction and HP reduction. This means the effects system relies on the caller to pre-compute vuln-adjusted damage. This is a separate code path from the combat engine and needs separate verification.

## Summary

| Issue | Severity | Status |
|-------|----------|--------|
| Split Vuln rounding error | Critical | **BUG** - causes 1-point errors |
| Torii before block | Critical | **BUG** - wrong HP loss with partial block |
| Torii lower bound | Not a bug | Correct (integer equivalence) |
| Intangible double-apply | Not a bug | Idempotent, functionally correct |
| Pen Nib/Weak order | Not a bug | Commutative multiplication |
| Slow power missing | Missing | Not implemented in `calculate_damage` |

## Recommended Fixes

1. **Vuln rounding**: Move Vuln into the single `calculate_damage()` float chain. Remove separate Vuln application in `_deal_damage_to_enemy()`.
2. **Torii ordering**: Apply Torii after block subtraction, on the `hp_loss` value, not on pre-block `final_damage`.
3. **Slow power**: Add Slow as a receiver multiplier in `calculate_damage()` or handle it in the combat engine.
