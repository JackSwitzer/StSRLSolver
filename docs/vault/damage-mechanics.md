# Slay the Spire Damage Mechanics

## Core Formula

```
Final Damage = floor((Base Damage + Additive Modifiers) * Multiplicative Modifiers)
```

## Order of Operations (Outgoing Damage)

1. **Base Damage**: Card's base value
2. **Additive**: Strength (+1/stack/hit), Vigor, Accuracy (Shivs), Wrist Blade (0-cost)
3. **Multiplicative** (all floor individually, stack multiplicatively):
   - Weak (attacker): x0.75
   - Vulnerable (target): x1.5
   - Wrath: x2
   - Divinity: x3
   - Pen Nib: x2
   - Phantasmal Killer: x2
   - Slow: x(1 + 0.1 * cards_played)

## Block Calculation

```
Final Block = floor((Base Block + Dexterity) * Frail Multiplier)
Frail = x0.75
```

**NOT affected by Dex/Frail:** Relics, potions, orbs, Metallicize, Plated Armor, After Image, Feel No Pain, Mental Fortress, Like Water, Nirvana, Rage, Entrench, Wallop

## Watcher Stances

| Stance | Damage Dealt | Damage Received | On Enter | On Exit |
|--------|-------------|-----------------|----------|---------|
| Neutral | x1 | x1 | - | - |
| Calm | x1 | x1 | - | +2 Energy |
| Wrath | x2 | x2 | - | - |
| Divinity | x3 | x1 | +3 Energy | Auto-exits turn start |

## Multi-Hit Attacks

Modifiers apply **per hit**:
- Strength: +1 per hit
- Pen Nib: first hit only
- Everything else: each hit

Heavy Blade: 14 + (3x or 5x upgraded) Strength, NOT per-hit

## Damage Reduction Pipeline

1. Block (subtract)
2. Buffer (negate hit, lose stack)
3. Torii (unblocked 2-5 -> 1, damage of 1 unchanged)
4. Intangible (all -> 1)
5. Tungsten Rod (HP loss -1)

## Thorns

- X damage back per hit (not per attack card)
- Triggers from Attacks only
- Sharp Hide (Guardian): per Attack card played

## Edge Cases

- **Minimum damage**: 0 (from negative Strength). Still triggers Thorns, not on-damage.
- **The Boot**: 1-4 unblocked -> 5. NOT 0, NOT non-attacks.
- **Intangible + Tungsten**: 1 -> 0 damage
- **Torii + Tungsten**: 2-5 unblocked -> 1 -> 0 (damage of 1 stays at 1, then -1 = 0)

## Code References (from decompilation)

### DamageInfo.applyPowers() Flow

**Location**: `com.megacrit.cardcrawl.cards.DamageInfo.class`

**Player attacking enemy**:
```
1. Player atDamageGive (Strength: +amount)
2. Player stance.atDamageGive (Wrath: x2, Divinity: x3)
3. Enemy atDamageReceive (Vulnerable: x1.5)
4. Player atDamageFinalGive
5. Enemy atDamageFinalReceive
6. floor(result), min 0
```

**Enemy attacking player**:
```
1. Enemy atDamageGive
2. Player atDamageReceive (Vulnerable)
3. Player stance.atDamageReceive (Wrath: x2)
4. Enemy atDamageFinalGive
5. Player atDamageFinalReceive
6. floor(result), min 0
```

### Power Implementations

**StrengthPower.class**:
```java
@Override
public float atDamageGive(float damage, DamageInfo.DamageType type) {
    if (type == DamageInfo.DamageType.NORMAL) {
        return damage + (float)this.amount;  // ADDITIVE
    }
    return damage;
}
```

**VulnerablePower.class**:
```java
@Override
public float atDamageReceive(float damage, DamageInfo.DamageType type) {
    if (type == DamageInfo.DamageType.NORMAL) {
        if (this.owner.isPlayer && AbstractDungeon.player.hasRelic("Odd Mushroom")) {
            return damage * 1.25f;  // Reduced vulnerability
        }
        if (!this.owner.isPlayer && AbstractDungeon.player.hasRelic("Paper Frog")) {
            return damage * 1.75f;  // Enhanced vulnerability
        }
        return damage * 1.5f;  // Default
    }
    return damage;
}
```

**WeakPower.class**:
```java
@Override
public float atDamageGive(float damage, DamageInfo.DamageType type) {
    if (type == DamageInfo.DamageType.NORMAL) {
        if (!this.owner.isPlayer && AbstractDungeon.player.hasRelic("Paper Crane")) {
            return damage * 0.6f;  // Enhanced weak
        }
        return damage * 0.75f;  // Default
    }
    return damage;
}
```

### Relic Damage Modifiers

| Relic | Effect | Where Applied |
|-------|--------|---------------|
| Odd Mushroom | Vulnerable on player = x1.25 | VulnerablePower.atDamageReceive |
| Paper Frog | Vulnerable on enemy = x1.75 | VulnerablePower.atDamageReceive |
| Paper Crane | Weak on enemy = x0.6 | WeakPower.atDamageGive |

## Implementation

Our implementation is in `core/calc/damage.py`. Key functions:

### Outgoing Damage (`calculate_damage`)
```python
def calculate_damage(
    base: int,
    strength: int = 0,
    vigor: int = 0,
    weak: bool = False,
    weak_paper_crane: bool = False,
    pen_nib: bool = False,
    double_damage: bool = False,
    stance_mult: float = 1.0,
    vuln: bool = False,
    vuln_paper_frog: bool = False,
    flight: bool = False,
    intangible: bool = False,
) -> int:
    # Order matches AbstractCard.calculateCardDamage():
    # 1. Base damage
    damage = float(base)
    # 2. Flat adds (Strength, Vigor)
    damage += strength + vigor
    # 3. Attacker multipliers (Pen Nib, Double Damage, Weak)
    if pen_nib: damage *= 2.0
    if double_damage: damage *= 2.0
    if weak: damage *= (0.60 if weak_paper_crane else 0.75)
    # 4. Stance multiplier
    damage *= stance_mult
    # 5. Defender multipliers (Vulnerable, Flight)
    if vuln: damage *= (1.75 if vuln_paper_frog else 1.5)
    if flight: damage *= 0.5
    # 6. Intangible cap
    if intangible and damage > 1: damage = 1.0
    # 7. Floor to int, minimum 0
    return max(0, int(damage))
```

### Block (`calculate_block`)
```python
def calculate_block(base: int, dexterity: int = 0, frail: bool = False) -> int:
    block = float(base) + dexterity
    if frail: block *= 0.75
    return max(0, int(block))
```

### Incoming Damage (`calculate_incoming_damage`)
```python
def calculate_incoming_damage(damage, block, is_wrath=False, vuln=False,
                               vuln_odd_mushroom=False, intangible=False,
                               torii=False, tungsten_rod=False) -> Tuple[int, int]:
    final_damage = float(damage)
    # 1. Wrath stance (2x incoming) - ONLY Wrath, not Divinity
    if is_wrath: final_damage *= 2.0
    # 2. Vulnerable
    if vuln: final_damage *= (1.25 if vuln_odd_mushroom else 1.5)
    final_damage = int(final_damage)
    # 3. Intangible
    if intangible and final_damage > 1: final_damage = 1
    # 4. Torii (damage 2-5 -> 1)
    if torii and 2 <= final_damage <= 5: final_damage = 1
    # 5. Block absorbs damage
    blocked = min(block, final_damage)
    hp_loss = final_damage - blocked
    # 6. Tungsten Rod (-1 HP loss)
    if tungsten_rod and hp_loss > 0: hp_loss = max(0, hp_loss - 1)
    return hp_loss, block - blocked
```

**Important**: Divinity (x3) only affects outgoing damage, NOT incoming damage. Only Wrath doubles both.

## Source Files

- `com.megacrit.cardcrawl.cards.DamageInfo` - Damage calculation
- `com.megacrit.cardcrawl.powers.StrengthPower` - Strength modifier
- `com.megacrit.cardcrawl.powers.VulnerablePower` - Vulnerable modifier
- `com.megacrit.cardcrawl.powers.WeakPower` - Weak modifier
- `com.megacrit.cardcrawl.relics.OddMushroom` - VULN_EFFECTIVENESS = 1.25f
- `com.megacrit.cardcrawl.relics.PaperFrog` - VULN_EFFECTIVENESS = 1.75f
- `com.megacrit.cardcrawl.relics.PaperCrane` - WEAK_EFFECTIVENESS = 0.6f
