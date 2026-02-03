# Damage-Triggered Relic Audit

Audit of Python engine relic implementations against decompiled Java source.

## Summary

| Relic | Java Hook | Python Implementation | Status | Notes |
|-------|-----------|----------------------|--------|-------|
| Torii | `onAttacked` | `calc/damage.py` + `combat_engine.py` + `handlers/combat.py` | ISSUE | Torii checks `damageAmount > 1 && damageAmount <= 5` in Java; Python checks `2 <= damage <= 5`. Equivalent but Python applies Torii BEFORE Intangible in combat_engine.py (wrong order). Java: Intangible is in `onAttackedToChangeDamage` which runs before `onAttacked`. |
| Tungsten Rod | `onLoseHpLast` | `calc/damage.py` + `combat_engine.py` + `handlers/combat.py` | OK | Correctly reduces HP loss by 1 (minimum 0). Applied after block. |
| Self-Forming Clay | `wasHPLost` | NOT IMPLEMENTED in combat | MISSING | Data definition exists in `relics.py` but no trigger in `combat_engine.py` or `handlers/combat.py`. Should apply `NextTurnBlockPower(3)` when HP is lost in combat. |
| Centennial Puzzle | `wasHPLost` | `handlers/combat.py:1178` | OK | Correctly draws 3 cards first time HP lost per combat. Uses counter to track once-per-combat. |
| Red Skull | `onBloodied`/`onNotBloodied` | NOT IMPLEMENTED in combat | MISSING | Data definition exists. Should grant +3 Strength when HP <= 50%, remove when healed above. Dynamic toggle not implemented. |
| Paper Crane | passive (`WEAK_EFFECTIVENESS=0.6f`) | `combat_engine.py:541`, `calc/damage.py` | OK | Correctly uses 0.60 multiplier (40% reduction from Weak instead of 25%). |
| Champion's Belt | `onTrigger(target)` | NOT IMPLEMENTED in combat | MISSING | Should apply 1 Weak whenever Vulnerable is applied. Java uses `onTrigger` called from `ApplyPowerAction` when Vulnerable is applied. |
| Horn Cleat | `atTurnStart` | `handlers/combat.py:437` | ISSUE | Python triggers on `turn == 2` then never again (correct). But Java uses counter: sets counter=0 at battle start, increments each turn, triggers at counter==2, then sets counter=-1 and grayscale=true. Python is functionally correct but doesn't match the counter mechanism. |
| Turnip | passive (Frail immunity) | Data only in `relics.py` | UNVERIFIED | Java has no hooks -- Frail immunity is checked in `ApplyPowerAction`. Need to verify Frail application code checks for Turnip. |
| Ginger | passive (Weak immunity) | Data only in `relics.py` | UNVERIFIED | Same as Turnip -- Weak immunity checked in `ApplyPowerAction`. |
| The Boot | `onAttackToChangeDamage` | Data only in `relics.py` | MISSING | Java: when player attacks and damage > 0 and < 5, set damage to 5. Not found in combat damage pipeline. |
| Fossilized Helix | `onAttacked` (implied, uses Buffer) | Data only in `relics.py` | MISSING | Should prevent all damage from first hit per combat (acts like 1 Buffer). |
| Runic Cube | `wasHPLost` | NOT IMPLEMENTED in combat | MISSING | Should draw 1 card whenever HP is lost. |

## Detailed Findings

### Torii - Order of Operations Issue

**Java** (`Torii.java:26-32`):
```java
public int onAttacked(DamageInfo info, int damageAmount) {
    if (info.owner != null && info.type != DamageInfo.DamageType.HP_LOSS
        && info.type != DamageInfo.DamageType.THORNS
        && damageAmount > 1 && damageAmount <= 5) {
        return 1;
    }
    return damageAmount;
}
```

**Python** (`combat_engine.py:561-567`):
```python
# Torii: reduce damage 2-5 to 1 (BEFORE Intangible)
if "Torii" in self.state.relics and 2 <= damage <= 5:
    damage = 1
# Intangible: cap damage to 1
if self.state.player.statuses.get("Intangible", 0) > 0 and damage > 1:
    damage = 1
```

Issues:
1. Java Torii excludes HP_LOSS and THORNS damage types; Python doesn't check damage type.
2. In `combat_engine.py`, Torii is applied BEFORE Intangible. In Java, `onAttackedToChangeDamage` (where Intangible lives) runs BEFORE `onAttacked` (where Torii lives). So the correct order is: Intangible first, then Torii.
3. In `calc/damage.py`, order is correct: Intangible (step 4), then Torii (step 5). But the comment says "Torii applies BEFORE block" which is correct.
4. In `handlers/combat.py`, Torii is applied to `final_damage` before `calculate_incoming_damage` which also applies Torii -- potential double application.

### Tungsten Rod - Correct

**Java** (`TungstenRod.java:22-27`):
```java
public int onLoseHpLast(int damageAmount) {
    if (damageAmount > 0) {
        return damageAmount - 1;
    }
    return damageAmount;
}
```

Python correctly applies this as the last step of HP loss calculation in all three locations.

### Self-Forming Clay - MISSING

**Java** (`SelfFormingClay.java:27-32`):
```java
public void wasHPLost(int damageAmount) {
    if (phase == COMBAT && damageAmount > 0) {
        addToTop(new ApplyPowerAction(player, player,
            new NextTurnBlockPower(player, 3, this.name), 3));
    }
}
```

No implementation found in combat handlers. Should gain 3 Block next turn whenever HP is lost.

### The Boot - MISSING

**Java** (`Boot.java:26-33`):
```java
public int onAttackToChangeDamage(DamageInfo info, int damageAmount) {
    if (info.owner != null && info.type != HP_LOSS && info.type != THORNS
        && damageAmount > 0 && damageAmount < 5) {
        return 5;
    }
    return damageAmount;
}
```

This is for OUTGOING player attack damage. When the player's attack would deal 1-4 damage, it deals 5 instead. Not implemented in the damage pipeline.

### Champion's Belt - MISSING

**Java** (`ChampionsBelt.java:28-31`):
```java
public void onTrigger(AbstractCreature target) {
    addToBot(new ApplyPowerAction(target, player, new WeakPower(target, 1, false), 1));
}
```

Called when Vulnerable is applied to a target. Not implemented.

### Red Skull - MISSING

**Java** (`RedSkull.java:31-72`): Dynamic strength toggle based on HP threshold (50%). Uses `onBloodied`/`onNotBloodied` hooks. Not implemented in combat.

### Horn Cleat - Functionally Correct

**Java** (`HornCleat.java:32-42`): Counter-based, triggers on turn 2 with 14 block. Python uses `turn == 2` check directly. Functionally equivalent, fires once per combat.

### Turnip/Ginger - Unverified

Both are passive relics with no hooks in Java. Frail/Weak immunity is checked at the point of debuff application. Need to verify that the Python debuff application code checks for these relics.

### Fossilized Helix - MISSING

Should prevent all damage from first attack per combat (similar to 1 stack of Buffer). Not implemented.

### Runic Cube - MISSING

Should draw 1 card on any HP loss. Not implemented.

## Priority Fixes

1. **Self-Forming Clay** - wasHPLost: gain 3 NextTurnBlock
2. **Runic Cube** - wasHPLost: draw 1 card
3. **The Boot** - onAttackToChangeDamage: minimum 5 damage on attacks
4. **Champion's Belt** - onTrigger: apply 1 Weak when Vulnerable applied
5. **Red Skull** - onBloodied/onNotBloodied: +3/-3 Strength at 50% HP
6. **Fossilized Helix** - onAttacked: prevent first damage per combat
7. **Torii order** - Fix Intangible/Torii ordering in combat_engine.py
8. **Torii damage type** - Should exclude HP_LOSS and THORNS damage types
9. **Turnip/Ginger** - Verify debuff immunity in apply code
